SECTION "Interupt Handlers", ROM0[$40]
; VBLANK handler
    jp VBlankHandler
    ds $48 - @

; LCD handler
    ; reti?
    ds $50 - @

; Timer handler
    ; reti?
    ds $58 - @

; Serial handler
    jp SerialHandler
    ds $60 - @

; Joypad handler
    ; reti?
