>[!TIP]
> Moved to [Codeberg](https://codeberg.org/SniverDaBest/ttp).

# Info
`ttp` is a simple utility to turn *almost* any text file into a picture.\
If you convert 500.6 MB of text files into a `.png`, the `.png` will be ~500.7 MB.

# How it works
Each byte is equal to a color value of a pixel.\
The information is stored in a GRBA format, and if there aren't enough characters to fill up the entire image, it will make the remaining space use pixels with A value `193`.\
\
If you wanted to encode `Hooray!`, it would be:
```
(111, 72, 111, 255), (97, 114, 121, 255), (0, 33, 0, 193), (0, 0, 0, 193)
 ^    ^    ^          ^    ^    ^             ^       ^________________^
 o    H    o          a    r    y             !       |Tells decoder to|
                                                      |strip 0 bytes in|
                                                      |these pixels.   |
                                                      ¯¯¯¯¯¯¯¯¯¯¯¯¯¯¯¯¯¯
```

If we didn't specify the alpha channel as `193` on the last two pixels, there would be extra `NULL` bytes at the end of the text file.

# TODO
- [X] Alpha channel for `NULL` bytes
- [X] TGA support
- [ ] JPG support
- [ ] Video support for large files
    - [ ] MP4 support
    - [ ] MPV support
    - [ ] MOV support
- [ ] Audio support
    - [ ] MP3 support
    - [ ] OGG support
    - [ ] WAV support
