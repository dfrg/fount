# Font enumeration, matching and fallback

This is a very early release of a cross-platform font enumeration/fallback library.
Right now, it supports querying fonts from a pre-baked static collection (Windows and macOS only for now) and from registered fonts scanned from user provided buffers.

More platforms and features to follow.

# Example

The only interesting thing to do with this library at the moment is to run the `itemize`
example with some text to confirm that the runs and reported fonts look reasonable. It will
report selected font family names for each run and nominal glyph identifiers for each character.

```
cargo run --example=itemize "العربية نص جميل. ܠܸܫܵܢܵܐ ܕܠܐ द क्विक ब्राउन फ़ॉक्स jumps over the lazy 🦸🏾‍♀️. Ꭰꮿꮩꮈ 1 Ꮒꭶꮣ, ߓߊߘߋ߲ ߕߐ߬ߡߊ ߟߎ߬" 
```

This should produce output similar to the following on Windows:
```
0: Arabic: ["Segoe UI"]
  "العربية نص جميل. "
  [2317, 2341, 2335, 2327, 2318, 2347, 2319, 3, 2343, 2331, 3, 2322, 2342, 2347, 2341, 17, 3]
1: Syriac: ["Segoe UI Historic"]
  "ܠ\u{738}ܫ\u{735}ܢ\u{735}ܐ ܕܠܐ "
  [2351, 2372, 2362, 2369, 2353, 2369, 2335, 3, 2340, 2351, 2335, 3]
2: Devanagari: ["Nirmala UI"]
  "द क\u{94d}विक ब\u{94d}राउन फ\u{93c}ॉक\u{94d}स "
  [265, 3, 248, 320, 281, 302, 248, 3, 271, 320, 275, 301, 234, 267, 3, 270, 325, 314, 248, 320, 285, 3]
3: Latin: ["Segoe UI"]
  "jumps over the lazy "
  [77, 88, 80, 83, 86, 3, 82, 89, 72, 85, 3, 87, 75, 72, 3, 79, 68, 93, 92, 3]
4: Emoji: ["Segoe UI Emoji"]
  "🦸🏾\u{200d}♀\u{fe0f}"
  [10910, 1074, 390, 8913, 665]
5: Latin: ["Segoe UI"]
  ". "
  [17, 3]
6: Cherokee: ["Gadugi"]
  "Ꭰꮿꮩꮈ 1 Ꮒꭶꮣ, "
  [249, 1138, 1116, 1083, 3, 20, 3, 283, 1065, 1110, 15, 3]
7: Nko: ["Ebrima"]
  "ߓߊߘߋ\u{7f2} ߕߐ\u{7ec}ߡߊ ߟߎ\u{7ec}"
  [1006, 970, 1026, 974, 1109, 3, 1014, 994, 1103, 1062, 970, 3, 1054, 986, 1103]
```

And on macOS:
```
0: Arabic: ["Geeza Pro"]
  "العربية نص جميل. "
  [240, 340, 300, 272, 242, 362, 246, 3, 348, 284, 3, 256, 344, 362, 340, 42, 3]
1: Syriac: ["Noto Sans Syriac"]
  "ܠ\u{738}ܫ\u{735}ܢ\u{735}ܐ ܕܠܐ "
  [156, 429, 276, 420, 180, 420, 9, 3, 63, 156, 9, 3]
2: Devanagari: ["Kohinoor Devanagari"]
  "द क\u{94d}विक ब\u{94d}राउन फ\u{93c}ॉक\u{94d}स "
  [68, 1, 51, 132, 79, 28, 51, 1, 73, 132, 77, 27, 6, 70, 1, 72, 133, 41, 51, 132, 82, 1]
3: Latin: ["System Font", "Helvetica"]
  "jumps over the lazy "
  [744, 890, 779, 838, 861, 3, 799, 922, 663, 843, 3, 878, 708, 663, 3, 756, 577, 952, 940, 3]
4: Emoji: ["Apple Color Emoji"]
  "🦸🏾\u{200d}♀\u{fe0f}"
  [2738, 922, 43, 45, 42]
5: Latin: ["System Font", "Helvetica"]
  ". "
  [1410, 3]
6: Cherokee: ["Galvji"]
  "Ꭰꮿꮩꮈ 1 Ꮒꭶꮣ, "
  [228, 437, 415, 382, 442, 15, 442, 262, 364, 409, 10, 442]
7: Nko: ["Noto Sans NKo"]
  "ߓߊߘߋ\u{7f2} ߕߐ\u{7ec}ߡߊ ߟߎ\u{7ec}"
  [27, 18, 32, 19, 58, 3, 29, 24, 52, 41, 18, 3, 39, 22, 52]
```
