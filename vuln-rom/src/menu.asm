include "constants.inc"

CURSOR_STATE_CONTINUE = 0
CURSOR_STATE_NEW_GAME = 1

CURSOR_POSITION_CONTINUE = $75
CURSOR_POSITION_NEW_GAME = $95

SECTION FRAGMENT "Menu", ROMX
RunMenu::
    xor a
    ld [rLCDC], a

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

    ; Reset VRAM bank
    ld a, 0
    ld [rVBK], a
.skipAttributeCopy

    ; Set cursor default location
    ld a, CURSOR_STATE_CONTINUE
    ld [menuCursorPosition], a

    ; Cursor Y
    xor a
    ld [shadowOAM], a

    ; Cursor X
    ld [shadowOAM + 1], a 
    
    ; Cursor tile index
    ld a, $91
    ld [shadowOAM + 2], a

    ; Cursor palette and attribute
    ld a, 0
    ld [shadowOAM + 3], a 

    ; Turn LDC on
    ld a, LCDC_DEFAULT
    ld [rLCDC], a
    ei

.loop
    ; Read inputs
    call ReadJoypad

    ; We handle the buttons first
    ld a, [joypadButtons]
    ld b, a
    ld a, [joypadButtonsOld]

    call GetNewlyPushedButtons

    bit 0, a
    jr nz, .a_pressed

    ; This time we load the Dpad
    ld a, [joypadDpad]
    ld b, a
    ld a, [joypadDpadOld]
    xor b

    call GetNewlyPushedButtons
    
    bit 2, a
    jr nz, .move_cursor
    bit 3, a
    jr nz, .move_cursor

    jr .inputsProcessed

.a_pressed
    ; Exits the menu
    jr HandleAPress
.move_cursor
    ld a, [menuCursorPosition]

    ; Flip the bit
    xor 1
    ld [menuCursorPosition], a
.inputsProcessed

    ; Move the cursor accordingly
    call MenuMoveCursor

    ; Lock so we wait for the frame to end
    ld a, 1
    ld [waitForFrame], a;
.waitForFrame
    ; Wait until waitForFrame = 0, which is set by the VBlank handler
    ld a, [waitForFrame]
    cp 0
    jr nz, .waitForFrame

    jr .loop

; Puts the cursor sprite at the right location
MenuMoveCursor:
    ld b, (CURSOR_POSITION_CONTINUE & $0F) << 3
    ld c, (CURSOR_POSITION_CONTINUE & $F0) >> 1

    ld a, [menuCursorPosition]
    cp CURSOR_STATE_CONTINUE
    jr z, :+

    ld b, (CURSOR_POSITION_NEW_GAME & $0F) << 3
    ld c, (CURSOR_POSITION_NEW_GAME & $F0) >> 1
:
    ld a, b
    ld [shadowOAM + 1], a

    ld a, c
    ld [shadowOAM], a

    ret

HandleAPress:
    ; Check if all buttons are pressed at once.
    ; This will help the bot get too the serial state faster
    xor %1111
    jr z, :++

    ; We chose the next step and exit this state
    ld b, GAMESTATE_MAP
    ld a, [menuCursorPosition]
    cp CURSOR_STATE_CONTINUE
    jr z, :+

    ld b, GAMESTATE_INPUT_MENU
:
    ld a, b
    ld [gameState], a

    ; We disable disable the arrow sprite
    ld a, 0
    ld [shadowOAM], a

    ; Exit the menu state
    ret
:
    ; We skip directly to the serial state
    ld a, GAMESTATE_SERIAL
    ld [gameState], a
    ret

SECTION FRAGMENT "Menu", ROMX, ALIGN[8]
menuTileMap:
INCBIN "res/menu_tilemap.bin"
.end

menuAttributes:
INCBIN "res/menu_attributes.bin"
.end
