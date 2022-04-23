SECTION "Interupt Handlers", ROM0[$40]
; VBLANK handler
    jp VBlankHandler
    ds $48 - @