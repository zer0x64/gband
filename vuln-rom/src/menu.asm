include "constants.inc"

SECTION FRAGMENT "Menu", ROMX
RunMenu::
    ; Copy the tile map
    ld de, menuTileMap
    ld hl, _SCRN0
    ld bc, menuTileMap.end - menuTileMap
    call CopyToVRAM

    ld a, [isCgb]
    cp 1
    jr nz, .skipAttributeCopy

    ; GDMA the attribute map
    ; Change VRAM bank
    ld a, 1
    ld [rVBK], a

    ld de, menuAttributes
    ld hl, _SCRN0
    ld bc, menuAttributes.end - menuAttributes
    call CopyToVRAM

    ; Start GDMA
    ld a, (((menuAttributes.end - menuAttributes) >> 4) - 1) | HDMA5F_MODE_GP
    ld [rHDMA5], a

    ; Reset VRAM bank
    ld a, 0
    ld [rVBK], a
.skipAttributeCopy
    ld a, LCDC_DEFAULT
    ld [rLCDC], a
    ei

.loop:
    ; Increment a test variable
    ld a, [testVariable]
    inc a
    ld [testVariable], a

    ; Lock so we wait for the frame to end
    ld a, 1
    ld [waitForFrame], a;
.waitForFrame
    ; Wait until waitForFrame = 0, which is set by the VBlank handler
    ld a, [waitForFrame]
    jr nz, .waitForFrame
    jr .loop


SECTION FRAGMENT "Menu", ROMX, ALIGN[8]
menuTileMap::
INCBIN "res/menu_tilemap.bin"
.end

menuAttributes::
INCBIN "res/menu_attributes.bin"
.end
