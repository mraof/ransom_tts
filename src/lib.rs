use std::collections::HashMap;
use csound::Csound;
use std::collections::hash_map::RandomState;
use std::fs::File;
use std::io::Write;
use std::io::BufRead;
use std::io::BufReader;
use std::process::Command;
use std::process::Stdio;
use std::process::ChildStdout;
use std::time::Duration;

const FLITE_VOICE_DIR: &str = "/usr/lib/flite/";

#[derive(Debug)]
pub enum TTS {
    Espeak,
    Festival,
    RHVoice,
    Flite,
    Mimic,
}
pub fn get_flite_voices() -> Vec<String> {
    std::fs::read_dir(FLITE_VOICE_DIR)
        .unwrap()
        .map(|f| f.unwrap().path().strip_prefix(FLITE_VOICE_DIR).unwrap().file_stem().unwrap().to_str().unwrap().to_string())
        .collect()
}

pub fn flite_tts(filename: &str, voice: &str, text: &str) {
    Command::new("flite")
        .arg("-t")
        .arg(text)
        .arg("-voicedir")
        .arg(FLITE_VOICE_DIR)
        .arg("-voice")
        .arg(voice)
        .arg("-o")
        .arg(filename)
        .output()
        .unwrap();
}

pub fn get_mimic_voices() -> Vec<String> {
    let output = Command::new("mimic")
        .arg("-lv")
        .output()
        .expect("Failed to execute process");

    String::from_utf8(output.stdout)
        .unwrap()
        .split(": ").nth(1)
        .unwrap()
        .split(' ')
        .filter(|s| !s.is_empty())
        .map(ToString::to_string)
        .collect()
}

pub fn mimic_tts(filename: &str, voice: &str, text: &str) {
    Command::new("mimic")
        .arg("-t")
        .arg(text)
        .arg("-voice")
        .arg(voice)
        .arg("-o")
        .arg(filename)
        .output()
        .unwrap();
}

pub fn get_rhvoice_voices() -> Vec<String> {
    vec![
        String::from("alan"),
        String::from("bdl"),
        String::from("clb"),
        String::from("evgeniy-eng"),
        String::from("lyubov"),
        String::from("slt"),
    ]
}

pub fn rhvoice_tts(filename: &str, voice: &str, text: &str) {
    let mut child = Command::new("RHVoice-test")
        .stdin(Stdio::piped())
        .arg("-p")
        .arg(voice)
        .arg("-o")
        .arg(filename)
        .spawn()
        .unwrap();
    let input = text.to_string();
    let mut stdin = child.stdin.take().expect("Failed to open stdin");
    std::thread::spawn(move || {
        stdin
            .write_all(input.as_bytes())
            .expect("Failed to write to stdin");
    });
    child.wait().unwrap();
}

pub fn get_espeak_voices() -> Vec<String> {
    let output = Command::new("espeak-ng")
        .arg("--voices=en")
        .output()
        .expect("Failed to execute process");

    String::from_utf8(output.stdout)
        .unwrap()
        .split('\n')
        .skip(1)
        .filter_map(|line| {
            line.split(' ')
                .filter(|s| !s.is_empty()).nth(4)
                .map(ToString::to_string)
        })
        .collect()
}

pub fn espeak_tts(filename: &str, voice: &str, text: &str) {
    let mut child = Command::new("espeak-ng")
        .stdin(Stdio::piped())
        .arg("-v")
        .arg(voice)
        .arg("-w")
        .arg(filename)
        .spawn()
        .unwrap();
    let input = text.to_string();
    let mut stdin = child.stdin.take().expect("Failed to open stdin");
    std::thread::spawn(move || {
        stdin
            .write_all(input.as_bytes())
            .expect("Failed to write to stdin");
    });
    child.wait().unwrap();
}

pub fn get_festival_voices() -> Vec<String> {
    let mut child = Command::new("festival_client")
        .arg("--withlisp")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    let mut stdin = child.stdin.take().expect("Failed to open stdin");
    std::thread::spawn(move || {
        stdin
            .write_all("(voice.list)\n".as_bytes())
            .expect("Failed to write to stdin");
    });
    std::thread::sleep(Duration::from_millis(500));

    let output = child.wait_with_output().expect("failed to wait on child");

    let output = String::from_utf8(output.stdout).unwrap();
    println!("{output}");
    output[1..(output.len() - 2)]
        .split(' ')
        .map(|s| s.to_string())
        .collect()
}

pub fn festival_tts(filename: &str, voice: &str, text: &str) {
    let mut child = Command::new("festival_client")
        .arg("--output")
        .arg(filename)
        .stdin(Stdio::piped())
        //.stdout(Stdio::piped())
        .spawn()
        .unwrap();
    let input = format!("(voice_{voice}) (tts_textall \"{text}\" nil)\n");
    let mut stdin = child.stdin.take().expect("Failed to open stdin");
    //let stdout = child.stdout.take().expect("Failed to open stdout");
    //festival_stdout(stdout);

    std::thread::spawn(move || {
        stdin
            .write_all(input.as_bytes())
            .expect("Failed to write to stdin");
    });
    child.wait().unwrap();
}

fn festival_stdout(stdout: ChildStdout) {
    let reader = BufReader::new(stdout);
    let mut lines = reader.lines().map(|l| l.unwrap());
    for line in lines.by_ref() {
      println!("Festival: {line}");
      if line.contains("Festival server started") {
        break;
      } else if line.contains("bind failed") {
        panic!("Failed to bind festival server");
      }
    }
    std::thread::spawn(move || {
      for line in lines {
        println!("Festival: {line}");
      }
    });

}

//TODO Don't load entire wav file just to get the length
pub fn wav_info(filename: &str) -> Option<(u32, usize)> {
    use wav::BitDepth::*;
    let mut file = File::open(filename).ok()?;
    let (header, data) = wav::read(&mut file).ok()?;
    let rate = header.sampling_rate;
    match data {
        Eight(v) => Some((rate, v.len() * 2)),
        Sixteen(v) => Some((rate, v.len() * 2)),
        TwentyFour(v) => Some((rate, v.len() * 2)),
        ThirtyTwoFloat(v) => Some((rate, v.len() * 2)),
        Empty => None,
    }
}

fn get_all_voices() -> Vec<(TTS, String)> {
    let mut festival_process = Command::new("festival").stdout(Stdio::piped()).arg("--server").spawn().unwrap();
    festival_stdout(festival_process.stdout.take().expect("Failed to open stdout"));

    let mut voices: Vec<_> = crate::get_festival_voices()
        .into_iter()
        .map(|v| (TTS::Festival, v))
        .collect();
    voices.extend(
        crate::get_espeak_voices()
            .into_iter()
            .map(|v| (TTS::Espeak, v)),
    );
    voices.extend(
        crate::get_rhvoice_voices()
            .into_iter()
            .map(|v| (TTS::RHVoice, v)),
    );
    festival_process
        .kill()
        .expect("We didn't need to kill it...");
    voices
}

pub struct RansomTTS {
    voices: Vec<(TTS, String)>
}

impl RansomTTS {
    pub fn new() -> RansomTTS {
        RansomTTS {
            voices: get_all_voices()
        }
    }

    pub fn tts(&self, text: &str) {
        std::fs::create_dir("wav").unwrap();
        let mut festival_process = Command::new("festival").stdout(Stdio::piped()).arg("--server").spawn().unwrap();
        festival_stdout(festival_process.stdout.take().expect("Failed to open stdout"));

        let words: Vec<_> = text.split(' ').map(|s| s.to_string()).collect();
        let unique_words: HashMap<String, usize, RandomState> = HashMap::from_iter(words.iter().cloned().enumerate().map(|(i, w)| (w, i)));
        let mut word_map = HashMap::new();
        let mut instruments = String::new();
        for (word, id) in &unique_words {
            let voice_id = word.chars().map(|c| c as usize).sum::<usize>() % self.voices.len();
            let (tts, voice) = &self.voices[voice_id];
            let filename = format!("wav/{id}.wav");
            match tts {
                TTS::Festival => festival_tts(&filename, voice, word),
                TTS::Espeak => espeak_tts(&filename, voice, word),
                TTS::RHVoice => rhvoice_tts(&filename, voice, word),
                TTS::Mimic => mimic_tts(&filename, voice, word),
                TTS::Flite => flite_tts(&filename, voice, word),
            }
            let fid = id + 2;
            if let Some((rate, samples)) = crate::wav_info(&filename) {
                instruments += &format!(include_str!("function.sco"), filename = filename, word = fid, samples = samples);
                let time = samples as f32 / rate as f32;
                word_map.insert(word, (id + 2, time));
            } else {
                instruments += &format!("f{} 0 1024 10 {} {} {}\n", fid, 0.3, 0.7, 0.8);
            }
        }
        festival_process
            .kill()
            .expect("We didn't need to kill it...");

        let csound = Csound::new();
        csound.set_output("output.wav", "wav", "NULL").unwrap();
        csound.message_string_callback(|mtype, message| println!("{mtype:?}: {message}"));
        csound.compile_orc(include_str!("word.orc")).unwrap();
        let mut score = String::new();
        score += include_str!("header.sco");
        score += &instruments;
        let mut beat = 0.0;
        for word in words.iter() {
            let pitch = word.chars().map(|c| c as u32).sum::<u32>() as f32 / (word.len() as f32) / ('m' as u32 as f32);
            if let Some((fid, time)) = word_map.get(word) {
                println!("{word}: {fid}");
                score += &format!(
                    include_str!("word.sco"),
                    beat = beat,
                    //beat = i,
                    pitch = pitch,
                    length = time,
                    word = fid,
                );
                beat += time;
            } else if let Some(id) = unique_words.get(word) {
                let fid = id + 2;
                let length = (word.len() as f32).sqrt();
                let start = ((pitch * 500.0 * length) as u32) % 2000;
                let mid = ((pitch * 2777.0 * length) as u32) % 2000;
                let end = ((pitch.sqrt() * 24885.0 * length) as u32) % 2000;
                score += &format!("i3 {beat} {length} {fid} {start} {mid} {end}\n");
                beat += length;
            }
        }
        score += "e";
        println!("{score}");
        csound.read_score(&score).unwrap();
        csound.start().unwrap();
        csound.perform();
        std::fs::remove_dir_all("wav").unwrap();
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn tts() {
        let text = "In a hole in the ground there lived a hobbit. Not a nasty, dirty, wet hole, filled with the ends of worms and an oozy smell, nor yet a dry, bare, sandy hole with nothing in it to sit down on or to eat: it was a hobbit-hole, and that means comfort. ";
        let tts = crate::RansomTTS::new();

        tts.tts(text);

    }
}
