include "constants.inc"

CURSOR_BASE_X = $3 
CURSOR_BASE_Y = $12
CURSOR_MAX_X = $6
CURSOR_MAX_Y = $6

MENU_STATE_NAME = 1
MENU_STATE_FLAG = 2
MENU_STATE_SAVE = 3

WORD_NAME_LOCATION = $2E
TEXT_ENTRY_LOCATION = $86

SECTION FRAGMENT "Input Menu", ROMX
RunInputMenu::
    ; Disable the PPU
    xor a
    ld [rLCDC], a

    ; We start without any scroll
    ld [shadowScrollX], a
    ld [shadowScrollY], a

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
    ld a, 0
    ld [menuCursorPosition], a

    ; Set the name and flag lengths
    ld a, 0
    ld [playerNameLengthRam], a
    ld [flagLengthRam], a
    ld [menuInputLength], a

    ; We start by entering the name
    ld a, MENU_STATE_NAME
    ld [menuState], a

    ; Cursor Y
    ld a, 16
    ld [shadowOAM], a

    ; Cursor X
    ld a, 8
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
    bit 1, a
    jr nz, .b_pressed

    ; This time we load the Dpad
    ld a, [joypadDpad]
    ld b, a
    ld a, [joypadDpadOld]
    xor b

    call GetNewlyPushedButtons
    call MenuHandleDpad
    jr .inputsProcessed

.a_pressed
    call HandleAPress
    jr .inputsProcessed
.b_pressed
    call DeleteChar
    jr .inputsProcessed

.inputsProcessed

    ; Check if we need to move on to the game map
    ld a, [menuState]
    cp MENU_STATE_SAVE
    jr nz, .notFinished

    ; We save data and exit this state
    ld a, GAMESTATE_MAP
    ld [gameState], a

    call SaveInputs
    ; We also disable the arrow sprite
    ld a, 0
    ld [shadowOAM], a

    ; Exit the menu state
    ret
.notFinished
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

    ; We are in vblank, copy text data
    call SetText
    jr .loop

; Handle Dpad outputs to move the cursor around
; @param a A bitfield of the newly pressed dpad directions
MenuHandleDpad:
    ; Switch case on the inputs
    bit 0, a
    jr nz, .right_pressed
    bit 1, a
    jr nz, .left_pressed
    bit 2, a
    jr nz, .up_pressed
    bit 3, a
    jr nz, .down_pressed

    ; Return if there is no input
    ret
.right_pressed
    ld b, $100 - CURSOR_MAX_X

    ld a, [menuCursorPosition]
    and $F
    cp CURSOR_MAX_X
    jr z, .dpad_processed

    ld b, 1
    jr .dpad_processed
.left_pressed
    ld b, CURSOR_MAX_X

    ld a, [menuCursorPosition]
    and $F
    cp 0
    jr z, .dpad_processed

    ld b, $FF
    jr .dpad_processed
.up_pressed
    ld b, CURSOR_MAX_Y << 4

    ld a, [menuCursorPosition]
    and $F0
    cp 0
    jr z, .dpad_processed

    ld b, $F0
    jr .dpad_processed
.down_pressed
    ld b, $100 - (CURSOR_MAX_Y << 4)

    ld a, [menuCursorPosition]
    and $F0
    cp CURSOR_MAX_Y << 4
    jr z, .dpad_processed

    ld b, $10
    jr .dpad_processed
.dpad_processed
    ; Add the offset
    ld a, [menuCursorPosition]
    add b
    ld [menuCursorPosition], a
    ret

; Puts the cursor sprite at the right location
MenuMoveCursor:
    ld a, [menuCursorPosition]

    ; Mask X only for now
    ld b, $0F
    and b

    ; Shift left because we hop 2 tiles at a time
    sla a
    
    ; Add base + 1
    ld b, CURSOR_BASE_X + 1
    add b

    ; Mutliply by 8 for subpixels
    sla a
    sla a
    sla a

    ; Copy to X position
    ld [shadowOAM + 1], a 

    ; Time to place Y
    ld a, [menuCursorPosition]

    ; Mask Y only for now
    ld b, $F0
    and b
    srl a
    srl a
    srl a
    srl a

    ; Add base + 2
    ld b, CURSOR_BASE_Y >> 1 + 2
    add b

    ; Mutliply by 8 for subpixels
    sla a
    sla a
    sla a

    ; Copy to Y position
    ld [shadowOAM], a 

    ret

SetText:
    ld a, [menuState]
    cp a, MENU_STATE_NAME
    jr z, .nameWordSet
    cp a, MENU_STATE_FLAG
    jr z, .flagWordSet
    ; If in an unknown state, skip this
    ret
.nameWordSet
    ld de, wordName
    ld bc, wordName.end - wordName
    
    jr .flagOrNameWordSet
.flagWordSet
    ld de, wordFlag
    ld bc, wordFlag.end - wordFlag
.flagOrNameWordSet
    ld hl, _SCRN0 + WORD_NAME_LOCATION
    call MemCpy

    ; Show the input
    ld c, flagRam.end - flagRam ; 16 characters

    ld a, [menuState]
    cp a, MENU_STATE_FLAG
    jr z, .printInput

    ; c: number of letters to show
    ld c, playerNameRam.end - playerNameRam ; 8 characters
.printInput
    ; b: Remaining letters
    ld a, [menuInputLength]
    ld b, a
    ; Pointer to the input buffer
    ld de, menuInput
    ; Pointer to the VRAM location
    ld hl, _SCRN0 + TEXT_ENTRY_LOCATION
.nameLoop
    ld a, b
    cp 0
    jr z, .nameLoopDefaultCharacter

    ; We copy one of the name bytes
    ld a, [de]
    ld [hli], a
    inc de
    dec b

    jr .nameDecAndLoop
.nameLoopDefaultCharacter
    ; We copy _
    ld a, $11; _
    ld [hli], a
.nameDecAndLoop
    dec c
    ld a, c
    cp 8
    jr z, .changeLine

    cp 0
    jr nz, .nameLoop
    ret
.changeLine
    push bc
    ld bc, $18
    add HL, bc
    pop bc

    ; Continue the loop
    jr .nameLoop


HandleAPress:
    ld a, [menuCursorPosition]

    cp $65
    jr z, .handleBack

    cp $66
    jr z, .handleEnd

    ; We add a character
    ld a, [menuInputLength]
    ld c, a

    ; We make sure the length is fine
    ld b, playerNameRam.end - playerNameRam
    ld a, [menuState]

    ; Choose the length according to the state
    cp MENU_STATE_NAME
    jr z, .lengthLoaded
    ld b, flagRam.end - flagRam

.lengthLoaded
    ; Compare to ensure we won't overflow
    ld a, c 
    cp b
    jr z, .ret

    ; Actually put the character in
    ld b, 0
    ld hl, menuInput
    add hl, bc

    push hl
    call GetAsciiCharacterFromCursor
    pop hl

    ld [hl], a

    ; Put the new length in
    ld a, [menuInputLength]
    inc a
    ld [menuInputLength], a
    ret
.handleBack
    jr DeleteChar
.handleEnd
    ; Don't handle it if the input field is empty
    ld a, [menuInputLength]
    cp 0
    jr z, .ret
    
    ld a, [menuState]
    cp MENU_STATE_NAME
    jr z, .handleEndName
    cp MENU_STATE_FLAG
    jr z, .handleEndFlag
    ret
.handleEndName    
    ; Copy the name to RAM
    ld a, [menuInputLength]
    ld c, a
    ld b, 0
    ld de, menuInput
    ld hl, playerNameRam
    call MemCpy

    ; Copy the name length to RAM
    ld a, [menuInputLength]
    ld [playerNameLengthRam], a

    ; Clear the data
    ld a, 0
    ld [menuInputLength], a

    ; Switch to flag input
    ld a, MENU_STATE_FLAG
    ld [menuState], a
    ret
.handleEndFlag
    ; Copy the flag in RAM
    ld de, menuInput
    ld hl, flagRam
    ld b, 0
    ld a, [menuInputLength]
    ld c, a
    call MemCpy

    ; Copy the flag length to RAM
    ld a, [menuInputLength]
    ld [flagLengthRam], a

    ; Switch to the next state
    ld a, MENU_STATE_SAVE
    ld [menuState], a
    ret
.ret
    ret

GetAsciiCharacterFromCursor:
    ld a, [menuCursorPosition]
    and $FF
    sla a

    ld h, 0
    ld l, a

    ld bc, menuTileMap
    add hl, bc

    ld bc, CURSOR_BASE_X + 1
    add hl, bc

    ld bc, CURSOR_BASE_Y << 4
    add hl, bc

    ld a, [hl]
    ret

DeleteChar:
    ld a, [menuInputLength]
    cp 0
    jr z, .ret
    dec a
    ld [menuInputLength], a
.ret
    ret

SaveInputs:
    ; Enable SRAM
    ld a, CART_SRAM_ENABLE
    ld [rRAMG], a

    ; Switch the bank
    ld a, BANK(saveHeader)
    ld [rRAMB], a

    ; Set the name length
    ld b, 0
    ld a, [playerNameLengthRam]

    ld [playerNameLengthSram], a

    ; Memcpy the name
    ld de, playerNameRam
    ld hl, playerNameSram
    ld b, 0
    ld c, a
    call MemCpy

    ; Set the flag length
    ld a, [flagLengthRam]
    ld [flagLengthSram], a

    ; Memcpy the flag
    ld de, flagRam
    ld hl, flagSram
    ld b, 0
    ld c, a
    call MemCpy

    ; Flag the save as valid
    ld a, 1
    ld [saveIsInitialized], a

    ; Sidable SRAM
    ld a, CART_SRAM_DISABLE
    ld [rRAMG], a

    ret

wordName:
    db "name?"
.end:
wordFlag:
    db "flag?"
.end:

SECTION FRAGMENT "Input Menu", ROMX, ALIGN[8]
menuTileMap:
INCBIN "res/input_menu_tilemap.bin"
.end

menuAttributes:
INCBIN "res/input_menu_attributes.bin"
.end
