RGB: MACRO
    dw (\3 << 10 | \2 << 5 | \1)
ENDM

BorderPalettes:
    INCBIN "res/sgb_border_tilemap.bin"
    
    ds $100
    
    RGB 30,29,29 ; PAL_SGB1
    RGB 25,22,25
    RGB 25,17,21
    RGB 24,14,12

    ds $18

    RGB 30,29,29 ; PAL_SGB2
    RGB 22,31,16
    RGB 27,20,6
    RGB 15,15,15

    ds $18

    RGB 30,29,29 ; PAL_SGB3
    RGB 31,31,17
    RGB 18,21,29
    RGB 15,15,15

    ds $18
.end

SGBBorderGraphics:
    INCBIN "res/ascii_tiles.bin"
.end

SGBSuperPalettes::
    ; *RANDOM* default palette
    RGB 27,31,31
    RGB 22,19,31
    RGB 21,7,15
    RGB 1,1,4
.end
