;sr = 16000
0dbfs = 1
instr 1  
  ipitch = p6  
  ifn = p5

  ktimewarp line 0, p3 * 2, p3
  ;kresample transeg 0.3, p3 / 3, 0, 1, p3 / 3 * 2, 1, 0.3
  kresample init 0.5 * ipitch
  ibeg = 0
  iwsize = 4410
  irandw = 882
  itimemode = 1
  ioverlap = p4

  a1 sndwarp 0.5, ktimewarp, kresample, ifn, ibeg, iwsize, irandw, ioverlap, 1, itimemode
  out a1
endin  

instr 2
  ifn = p4

  a1 loscil 0.5, 1, ifn, 1, 0
  out a1
endin  

instr 3
  ifn = p4  
  kamp = 0.5
  kfreq expseg p5, 1, p6, 1, p7
  k1 poscil3 100, kfreq, ifn  
  k2 poscil3 200, k1, ifn  
  k3 poscil3 300, k2, ifn  
  k4 poscil3 kfreq, k2, ifn  
  ;k5 poscil3 k3, k4, ifn  
  a1 poscil3 kamp, k4, ifn  
  out a1  
endin  