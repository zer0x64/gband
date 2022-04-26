include "constants.inc"

SECTION FRAGMENT "Game Loop", ROMX
RunGame::
    xor a
    ld [rLCDC], a

    ; Copy the tile map
    ld de, mapTileMap
    ld hl, _SCRN0
    ld bc, mapTileMap.end - mapTileMap
    call CopyToVRAM

    ld a, [isCgb]
    cp 1
    jr nz, .skipAttributeCopy

    ; GDMA the attribute map
    ; Change VRAM bank
    ld a, 1
    ld [rVBK], a

    ld de, mapAttributes
    ld hl, _SCRN0
    ld bc, mapAttributes.end - mapAttributes
    call CopyToVRAM

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

SECTION FRAGMENT "Game Loop", ROMX, ALIGN[8]
mapTileMap::
INCBIN "res/map_tilemap.bin"
.end

mapAttributes::
INCBIN "res/map_attributes.bin"
.end
