include "constants.inc"

CHARACTER_SCREEN_POSITION_X = 80 + 8
CHARACTER_SCREEN_POSITION_Y = 72 + 16

; The position represents the middle of the tile
CHARACTER_DEFAULT_POSITION_X = 128
CHARACTER_DEFAULT_POSITION_Y = 112

MAX_SCROLL_X = 256 - 160
MAX_SCROLL_Y = 256 - 144

; The character hitbox size is 4x4
HITBOX_SIZE = 4

SECTION FRAGMENT "Game Loop", ROMX
RunGame::
    ; Disable the PPU
    xor a
    ld [rLCDC], a

    ; We start without any scroll
    ld [shadowScrollX], a
    ld [shadowScrollY], a

    ; Sets the default position of the character
    ld a, CHARACTER_DEFAULT_POSITION_X
    ld [characterPositionX], a

    ld a, CHARACTER_DEFAULT_POSITION_Y
    ld [characterPositionY], a

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

    ; Character Y
    ld a, CHARACTER_SCREEN_POSITION_Y
    ld [shadowOAM], a

    ; Cursor X
    ld a, CHARACTER_SCREEN_POSITION_X
    ld [shadowOAM + 1], a 
    
    ; Character default tile index
    ld a, 0
    ld [shadowOAM + 2], a

    ; character palette and attribute
    ld a, 1
    ld [shadowOAM + 3], a 

    ; Enable PPU
    ld a, LCDC_DEFAULT
    ld [rLCDC], a
    ei

.loop:
    ; We update the joypad state
    call ReadJoypad

    ; We move the character according to the inputs
    call MoveCharacter

    ; This calculate the screen scroll
    call CalculateScroll

    ; This calculate the sprite position on the screen
    ; Normally the sprite will be at the center of the screen,
    ;   but if there's a scroll lock the sprite can move around freely
    call CalculateSpriteScreenPosition

    ; Lock so we wait for the frame to end
    ld a, 1
    ld [waitForFrame], a;
.waitForFrame
    ; Wait until waitForFrame = 0, which is set by the VBlank handler
    ld a, [waitForFrame]
    cp a, 0
    jr nz, .waitForFrame

    jr .loop

MoveCharacter:
    ld a, [joypadDpad]

    ; Check X movement
    ld b, $00

    bit 0, a
    jr z, :+
    bit 1, a
    jr z, :++
    jr :+++
:
    ; Right
    ld b, $01
    jr :++
:
    ; Left
    ld b, $FF
    jr :+
:
    ; Apply X movement
    ld a, [characterPositionX]
    add a, b
    ld [characterPositionX], a

    ; Check Y movement
    ld a, [joypadDpad]

    ld b, $00

    bit 2, a
    jr z, :+
    bit 3, a
    jr z, :++
    jr :+++
:
    ; Up
    ld b, $FF
    jr :++
:
    ; Down
    ld b, $01
    jr :+
:
    ld a, [characterPositionY]
    add a, b
    ld [characterPositionY], a

    ret

CalculateScroll:
    ; Here we calculate X scroll
    ld a, [characterPositionX]
    sub a, 80 + 4

    ; Check if we're on the edge for X-
    jr c, :+

    ; Check if we're on the edge for X+
    ld b, a
    ld a, MAX_SCROLL_X
    sub b
    jr c, :++

    ; We got scrollX in B, we can apply it
    jr .applyScrollX
:
    ; Screen is locked on X-
    ld b, 0
    jr .applyScrollX
:
    ; Screen is locked on X+
    ld b, MAX_SCROLL_X
    jr .applyScrollX
.applyScrollX
    ld a, b
    ld [shadowScrollX], a

    ; Here we calculate Y scroll
    ld a, [characterPositionY]
    sub a, 72 + 4

    ; Check if we're on the edge for Y-
    jr c, :+

    ; Check if we're on the edge for Y+
    ld b, a
    ld a, MAX_SCROLL_Y
    sub b
    jr c, :++

    ; We got scrollY in B, we can apply it
    jr .applyScrollY
:
    ; Screen is locked on Y-
    ld b, 0
    jr .applyScrollY
:
    ; Screen is locked on Y+
    ld b, MAX_SCROLL_Y
    jr .applyScrollY
.applyScrollY
    ld a, b
    ld [shadowScrollY], a

    ret

CalculateSpriteScreenPosition:
    ; We start by calculating the X position
    ld a, [shadowScrollX]
    cp 0
    jr z, :+
    cp MAX_SCROLL_X
    jr z, :++

    ld a, CHARACTER_SCREEN_POSITION_X
    jr .loadX
:
    ; Screen is locked to the left, so the sprite can go move freely on X
    ld a, [characterPositionX]

    ; Add 8 because of the offset in OAM, remove 4 to get the left side
    add 8 - 4

    jr .loadX
:
    ; Screen is locked to the right, so the sprite can go move freely on X with an offset
    ld a, [characterPositionX]

    ; Add 8 because of the offset in OAM, remove 4 to get the left side
    sub (MAX_SCROLL_X - (8 - 4))

    jr .loadX
.loadX
    ld [shadowOAM + 1], a

    ; We now calculate the Y position
    ld a, [shadowScrollY]
    cp 0
    jr z, :+
    cp MAX_SCROLL_Y
    jr z, :++

    ld a, CHARACTER_SCREEN_POSITION_Y
    jr .loadY
:
    ; Screen is locked to the bottom, so the sprite can go move freely on Y
    ld a, [characterPositionY]

    ; Add 16 because of the offset in OAM, remove 4 to get the top side
    add (16 - 4)

    jr .loadY
:
    ; Screen is locked to the top, so the sprite can go move freely on Y with an offset
    ld a, [characterPositionY]

    ; Add 16 because of the offset in OAM, remove 4 to get the top side
    sub (MAX_SCROLL_Y - (16 - 4))

    jr .loadY
.loadY
    ld [shadowOAM], a
    ret

SECTION FRAGMENT "Game Loop", ROMX, ALIGN[8]
mapTileMap:
INCBIN "res/map_tilemap.bin"
.end

mapAttributes:
INCBIN "res/map_attributes.bin"
.end

mapLogic:
INCBIN "res/map_logic.bin"
.end
