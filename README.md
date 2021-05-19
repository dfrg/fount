## Font enumeration, matching and fallback

This is a very early release of a cross-platform font enumeration/fallback library.
Right now, it supports querying font families from a pre-baked static collection (Windows only for now) and from registered font files scanned from user provided buffers.

More platforms and features to follow in the coming weeks.

## Example

The only interesting thing to do with this library at the moment is to run the `itemize`
example with some text to confirm that the runs and reported fonts look reasonable:

```
cargo run --example=itemize "العربية نص جميل. ܠܸܫܵܢܵܐ ܕܠܐ द क्विक ब्राउन फ़ॉक्स jumps over the lazy 🦸🏾‍♀️. Ꭰꮿꮩꮈ 1 Ꮒꭶꮣ, ߓߊߘߋ߲ ߕߐ߬ߡߊ ߟߎ߬" 
```

This should produce output similar to the following:
```
0: Arabic: ["Segoe UI"]
  "العربية نص جميل. "
1: Syriac: ["Segoe UI Historic"]
  "ܠ\u{738}ܫ\u{735}ܢ\u{735}ܐ ܕܠܐ "
2: Devanagari: ["Nirmala UI"]
  "द क\u{94d}विक ब\u{94d}राउन फ\u{93c}ॉक\u{94d}स "
3: Latin: ["Segoe UI"]
  "jumps over the lazy "
4: Emoji: ["Segoe UI Emoji"]
  "🦸🏾\u{200d}♀\u{fe0f}"
5: Latin: ["Segoe UI"]
  ". "
6: Cherokee: ["Gadugi"]
  "Ꭰꮿꮩꮈ 1 Ꮒꭶꮣ, "
7: Nko: ["Ebrima"]
  "ߓߊߘߋ\u{7f2} ߕߐ\u{7ec}ߡߊ ߟߎ\u{7ec}"
```
