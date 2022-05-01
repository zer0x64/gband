include "constants.inc"

CHARACTER_SCREEN_POSITION_X = 80 + 8
CHARACTER_SCREEN_POSITION_Y = 72 + 16

; The position represents the middle of the tile
CHARACTER_DEFAULT_POSITION_X = 128
CHARACTER_DEFAULT_POSITION_Y = 112

; The direction the character faces
CHARACTER_DIRECTION_DOWN = $00
CHARACTER_DIRECTION_RIGHT = $10
CHARACTER_DIRECTION_LEFT = $20
CHARACTER_DIRECTION_UP = $30

MAX_SCROLL_X = 256 - 160
MAX_SCROLL_Y = 256 - 144

MAP_ENTITY_EMPTY = 0
MAP_ENTITY_SOLID = 1
MAP_ENTITY_NPC = 2
MAP_ENTITY_FLAG = 3
MAP_ENTITY_CHICKEN = 4

MAP_STATE_RUNNING = 0
MAP_STATE_TALKING = 1
MAP_STATE_NPC = 2
MAP_STATE_EXITING = 3

TEXTBOX_LINE_LENGTH = 18

DEFAULT_NPC_CURSOR_POSITION = $6B

; The character hitbox size is 4x4
HITBOX_SIZE = 6

INTERACTION_RANGE = 8

; How many frames pass between each frame of walk cycle animation
WALK_ANIMATION_SPEED = 8

SECTION FRAGMENT "Game Loop", ROMX
RunGame::
    ; Disable the PPU
    xor a
    ld [rLCDC], a

    ; We start without any scroll
    ld [shadowScrollX], a
    ld [shadowScrollY], a

    ; We start with the window off
    ld [shadowWindow], a

    ; We set the textbox text to be empty
    call ClearTextboxText

    ; We set the window position
    ld a, 7
    ld [rWX], a
    ld a, 144 - 40
    ld [rWY], a

    ; Sets the default position of the character
    ld a, CHARACTER_DEFAULT_POSITION_X
    ld [characterPositionX], a

    ld a, CHARACTER_DEFAULT_POSITION_Y
    ld [characterPositionY], a

    ; Character starts facing down
    ld a, CHARACTER_DIRECTION_DOWN
    ld [characterDirection], a

    ld [animationCycleTimer], a

    ld a, DEFAULT_NPC_CURSOR_POSITION
    ld [npcCursorPosition], a

    ; Copy the tile map
    ld de, mapTileMap
    ld hl, _SCRN0
    ld bc, mapTileMap.end - mapTileMap
    call CopyToVRAM

    ; Copy the window tile map
    ld de, mapWindowTileMap
    ld hl, _SCRN1
    ld bc, mapWindowTileMap.end - mapWindowTileMap
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

    ; GDMA the window attribute map
    ld de, mapWindowAttributes
    ld hl, _SCRN1
    ld bc, mapWindowAttributes.end - mapWindowAttributes
    call CopyToVRAM

    ; Reset VRAM bank
    ld a, 0
    ld [rVBK], a
.skipAttributeCopy

    ; Load game state
    ld a, MAP_STATE_RUNNING
    ld [mapState], a

    ; Character Y
    ld a, CHARACTER_SCREEN_POSITION_Y
    ld [shadowOAM], a

    ; Cursor X
    ld a, CHARACTER_SCREEN_POSITION_X
    ld [shadowOAM + 1], a 
    
    ; Character default tile index
    ld a, CHARACTER_DIRECTION_DOWN
    ld [shadowOAM + 2], a

    ; character palette and attribute
    ld a, 1
    ld [shadowOAM + 3], a 

    ; Enable PPU
    ld a, LCDC_DEFAULT
    ld [rLCDC], a
    ei

.loop:
    ; We check the map state to decide what to do
    ld a, [mapState]

    cp MAP_STATE_RUNNING
    jr z, .mainLoop

    cp MAP_STATE_EXITING
    jr z, .exit

    cp MAP_STATE_TALKING
    jr z, .talking

    cp MAP_STATE_NPC
    jr z, .talkingToNpc
.exit
    ; Turn PPU off and exit
    xor a
    ld [rLCDC], a
    ret
.talking
    ; We update the joypad state
    call ReadJoypad

    ; We check if a has newly been pressed
    ld a, [joypadButtons]
    ld b, a
    ld a, [joypadButtonsOld]

    call GetNewlyPushedButtons

    and a, %11
    cp 0

    jr z, :+

    ; A or B has been pressed, exit the state
    ld a, MAP_STATE_RUNNING
    ld [mapState], a

    ; Disable the window
    ld a, 0
    ld [shadowWindow], a

:
    jr .render
.talkingToNpc
    ; interaction with serial NPC
    call NpcInteraction

    jr .render
.mainLoop
    ; We update the joypad state
    call ReadJoypad

    ; We move the character according to the inputs
    call MoveCharacter

    ; We change the character direction to match the inputs
    call ChangeCharacterDirection

    ; We update the character's animation cycle
    call SetAnimationCycle

    ; We load the character sprite
    ld a, [characterDirection]
    ld [shadowOAM + 2], a

    ; This calculate the screen scroll
    call CalculateScroll

    ; This calculate the sprite position on the screen
    ; Normally the sprite will be at the center of the screen,
    ;   but if there's a scroll lock the sprite can move around freely
    call CalculateSpriteScreenPosition

    ; Check if there is an interaction to process
    call CheckInteraction

.render
    ; Lock so we wait for the frame to end
    ld a, 1
    ld [waitForFrame], a;
.waitForFrame
    ; Wait until waitForFrame = 0, which is set by the VBlank handler
    ld a, [waitForFrame]
    cp a, 0
    jr nz, .waitForFrame
 
    ; Print window
    call DrawWindow
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
    ld a, [characterDirection]
    and %00001111
    or CHARACTER_DIRECTION_RIGHT
    ld [characterDirection], a
    jr :++
:
    ; Left
    ld b, $FF
    ld a, [characterDirection]
    and %00001111
    or CHARACTER_DIRECTION_LEFT
    ld [characterDirection], a
    jr :+
:
    ; Apply X movement
    ld a, [characterPositionX]
    add a, b

    ; Store the value
    ld [localVariables], a

    ; We check if the new position is valid
    ld e, a
    ld a, [characterPositionY]
    ld d, a

    call CheckCollision

    ; If the tile is not valid, we don't commit the new position
    cp 0
    jr nz, .y_movement

    ld a, [localVariables]
    ld [characterPositionX], a

.y_movement
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
    ld a, [characterDirection]
    and %00001111
    or CHARACTER_DIRECTION_UP
    ld [characterDirection], a
    jr :++
:
    ; Down
    ld b, $01
    ld a, [characterDirection]
    and %00001111
    or CHARACTER_DIRECTION_DOWN
    ld [characterDirection], a
    jr :+
:
    ld a, [characterPositionY]
    add a, b

    ; Store the value
    ld [localVariables], a

    ; We check if the new position is valid
    ld d, a
    ld a, [characterPositionX]
    ld e, a

    call CheckCollision

    ; If the tile is not valid, we don't commit the new position
    cp 0
    ret nz

    ld a, [localVariables]
    ld [characterPositionY], a

    ret

ChangeCharacterDirection:
    ; Here we change the direction the character is facing
    ld a, [characterDirection]
    ld [shadowOAM + 2], a

    ; mask so I just get the direction and not the bits for the animation cycle
    and %00110000

    ; check if pressing up
    cp CHARACTER_DIRECTION_UP
    jr z, :+

    ; check if pressing down
    cp CHARACTER_DIRECTION_DOWN
    jr z, :++

    jr .resetFlip
    
: ; if pressing up
    ld a, [joypadDpad]

    ; check if also pressing right
    bit 0, a
    jr z, .applyFlip

    jr .resetFlip

: ; if pressing down
    ld a, [joypadDpad]
    
    ; check if also pressing left
    bit 1, a
    jr z, .applyFlip

    jr .resetFlip
.applyFlip
    ld a, [shadowOAM + 3]
    set 5, a
    jr .applyNewDirection

.resetFlip
    ld a, [shadowOAM + 3]
    res 5, a

.applyNewDirection
    ld [shadowOAM + 3], a

    ret

SetAnimationCycle:
    ld a, [joypadDpad]
    ld b, %00001111
    and b
    
    ; if no inputs, reset animation cycle
    cp b
    jr z, .resetCycle

    ; check if the timer is ellapsed
    ld a, [animationCycleTimer]
    cp WALK_ANIMATION_SPEED
    jr c, :+
    jr z, :++

    ; this means it incremented over the timer limit somehow... reset
    jr .resetCycle

: ; timer not ellapsed, increment by 1
    inc a
    ld [animationCycleTimer], a

    ret
: ; timer ellapsed, change to next frame of animation
    ; reset timer
    ld a, 0
    ld [animationCycleTimer], a

    ; we change bit 1 so it switches to the other frame of animation
    ld a, [characterDirection]
    xor $01
    ld [characterDirection], a

    and $01

    cp $00
    jr z, :+

    ret
    
: ; back to frame 0 of animation. switch leg
    ld a, [characterDirection]
    xor %00001000
    ld [characterDirection], a

    ret

.resetCycle
    ld a, $00
    ld [animationCycleTimer], a

    ld a, [characterDirection]
    and %11110000
    ld [characterDirection], a
    
    ret

NpcInteraction:
    ; We update the joypad state
    call ReadJoypad

    ; We check newly pressed buttons
    ld a, [joypadButtons]
    ld b, a
    ld a, [joypadButtonsOld]

    call GetNewlyPushedButtons

    bit 1, a
    jr nz, :++
    bit 0, a
    jr nz, :+

    ; now check newly pressed dpad inputs
    ld a, [joypadDpad]
    ld b, a
    ld a, [joypadDpadOld]

    call GetNewlyPushedButtons

    ; we check if left or right are pressed
    and %00000011
    cp 0

    ret z

    ld a, [npcCursorPosition]
    xor %00001000
    ld [npcCursorPosition], a
    
    ret

: ; A pressed
    ld a, [npcCursorPosition]

    cp DEFAULT_NPC_CURSOR_POSITION ; here I kinda just assume the default is No...
    jr nz, :++

: ; No selected (or B pressed), exiting dialog
    ld a, MAP_STATE_RUNNING
    ld [mapState], a
    
    ; Disable the window
    ld a, 0
    ld [shadowWindow], a

    ; reset cursor position for next time
    ld a, DEFAULT_NPC_CURSOR_POSITION
    ld [npcCursorPosition], a

    ret

: ; Yes selected, confirmed dialog
    call ClearTextboxText

    ld a, MAP_STATE_EXITING
    ld [mapState], a

    ld a, GAMESTATE_SERIAL
    ld [gameState], a

    xor a
    ld [shadowWindow], a
    
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

; Check if the point collides with a solid object
; @param d Y position of the point to check
; @param e X position oof the point to check
; returns a == 0 if the player can move there
CheckCollision:
    ; Check top left
    ld h, d
    ld l, e

    ld a, d
    sub HITBOX_SIZE / 2
    ld d, a

    ld a, e
    sub HITBOX_SIZE / 2
    ld e, a

    push hl
    call GetLogicTile
    pop hl

    ; If a is not zero, break
    cp MAP_ENTITY_EMPTY
    jr nz, .break

    ; Check bottom left
    ld d, h
    ld e, l

    ld a, d
    add HITBOX_SIZE / 2
    ld d, a

    ld a, e
    sub HITBOX_SIZE / 2
    ld e, a

    push hl
    call GetLogicTile
    pop hl

    ; If a is not zero, break
    cp MAP_ENTITY_EMPTY
    jr nz, .break

    ; Check top right
    ld d, h
    ld e, l

    ld a, d
    sub HITBOX_SIZE / 2
    ld d, a

    ld a, e
    add HITBOX_SIZE / 2
    ld e, a

    push hl
    call GetLogicTile
    pop hl

    ; If a is not zero, break
    cp MAP_ENTITY_EMPTY
    jr nz, .break

    ; Check bottom right
    ld d, h
    ld e, l

    ld a, d
    add HITBOX_SIZE / 2
    ld d, a

    ld a, e
    add HITBOX_SIZE / 2
    ld e, a

    push hl
    call GetLogicTile
    pop de

    ; If a is not zero, break
    cp MAP_ENTITY_EMPTY
    jr nz, .break

    ; Every checks has passed, return 1
    xor a
    ret

.break
    ld a, 1
    ret

; Checks if the player presses A on an interactable tile
CheckInteraction::
    ; We handle the buttons first
    ld a, [joypadButtons]
    ld b, a
    ld a, [joypadButtonsOld]

    call GetNewlyPushedButtons

    ; We only check for the a button
    bit 0, a
    ; We immediately return if a is not newly pressed
    ret z

    ld a, [characterDirection]
    and %00110000
    cp a, CHARACTER_DIRECTION_LEFT
    jr z, .facingLeft
    cp a, CHARACTER_DIRECTION_RIGHT
    jr z, .facingRight
    cp a, CHARACTER_DIRECTION_UP
    jr z, .facingUp
    cp a, CHARACTER_DIRECTION_DOWN
    jr z, .facingDown

    ; Load the Y offset in d and the X offset in e
.facingLeft
    ld d, 0
    ld e, -INTERACTION_RANGE
    jr .checkInteraction
.facingRight
    ld d, 0
    ld e, INTERACTION_RANGE
    jr .checkInteraction
.facingUp
    ld d, -INTERACTION_RANGE
    ld e, 0
    jr .checkInteraction
.facingDown
    ld d, INTERACTION_RANGE
    ld e, 0
    jr .checkInteraction
.checkInteraction
    ; Compute the Y position to check
    ld a, [characterPositionY]
    add d
    ld d, a

    ; Compute the X position to check
    ld a, [characterPositionX]
    add e
    ld e, a

    ; Get the tile value
    call GetLogicTile

    cp MAP_ENTITY_FLAG
    jr z, .handleFlagInteraction

    cp MAP_ENTITY_CHICKEN
    jr z, .handleChickenInteraction

    cp MAP_ENTITY_NPC
    jr z, .handleNpcInteraction

    ; Other tiles don't do anything when interacted with
    ret

.handleFlagInteraction
    ld a, MAP_STATE_TALKING
    ld [mapState], a

    ; We show the window
    ld a, 1
    ld [shadowWindow], a

    ; We clear the textbox string
    call ClearTextboxText

    ; We put the chicken text in
    ld de, flagRam
    ld hl, textboxText
    ld b, 0
    ld a, [flagLengthRam]
    ld c, a
    call MemCpy
    
    ret
.handleChickenInteraction
    ld a, MAP_STATE_TALKING
    ld [mapState], a

    ; We show the window
    ld a, 1
    ld [shadowWindow], a

    ; We clear the textbox string
    call ClearTextboxText

    ; We put the chicken text in
    ld de, chickenText
    ld hl, textboxText
    ld bc, chickenText.end - chickenText
    call MemCpy

    ret

.handleNpcInteraction
    ld a, MAP_STATE_NPC
    ld [mapState], a

    ; We show the window
    ld a, 1
    ld [shadowWindow], a

    ; We clear the textbox string
    call ClearTextboxText

    ; We put the chicken text in
    ld de, npcText
    ld hl, textboxText
    ld bc, npcText.end - npcText
    call MemCpy
    
    ret

; Check if the point collides with a solid object
; @param d Y position of the point to check
; @param e X position oof the point to check
; returns a The enum value of the object
GetLogicTile:
    ; Divides each componnents by 8 to remove subpixels
    ld a, d
    and a, %11111000
    ld d, a

    ; Adresses are on 10 bits, 5 for X and 5 for Y
    ; The 5 X bits are the lower 5 bits of tthe lower register(e in the case of de)
    srl e
    srl e
    srl e

    ; The 5 bits of Y are a bit more complicated:
    ; The 3 lower bits of y are the 3 higher bits of e...
    sla a
    sla a
    or e

    ld e, a

    ; ... while the 2 higher bits of y are the 2 lower bits of d
    ld a, d
    srl a
    srl a
    srl a
    srl a
    srl a
    srl a
    ld d, a

    ; We load the logic map address and add the calculated offset
    ld hl, mapLogic
    add hl, de

    ; We fetch the logic byte
    ld a, [hl]

    ret

DrawWindow:
    ld a, [shadowWindow]
    cp 0
    jr z, .windowOff

    ; Turn window on
    ld a, [rLCDC]
    set 5, a
    ld [rLCDC], a

    ; We draw the two lines of text on the window
    ld de, textboxText
    ld hl, _SCRN1 + $21
    ld bc, TEXTBOX_LINE_LENGTH
    call MemCpy

    ld hl, _SCRN1 + $41
    ld bc, TEXTBOX_LINE_LENGTH
    call MemCpy

    ; Check if we're talking to the NPC for the 3rd line
    ld a, [mapState]
    cp MAP_STATE_NPC
    jr z, :+
    
    ; Not talking to NPC
    ; Clear the yes/no text
    xor a
    ld hl, _SCRN1 + $61
    ld bc, TEXTBOX_LINE_LENGTH
    call MemSet

    ret
: ; talking to NPC
    ld de, yesNoText
    ld hl, _SCRN1 + $61
    ld bc, yesNoText.end - yesNoText
    call MemCpy

    ; place cursor in NPS window
    ld a, [npcCursorPosition]
    ld hl, _SCRN1
    ld l, a

    ; cursor tile index
    ld a, $91
    ld [hl], a

    ret

.windowOff
    ; Turn window off
    ld a, [rLCDC]
    res 5, a
    ld [rLCDC], a
    ret

ClearTextboxText:
    ; We set the textbox text to be empty
    xor a
    ld hl, textboxText
    ld bc, textboxText.end - textboxText
    call MemSet

    ret

chickenText:
DB "poc poc", $01
.end

npcText:
DB "Connect to the    other side?"
.end

yesNoText:
DB "    yes     no"
.end

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

mapWindowTileMap:
INCBIN "res/map_window_tilemap.bin"
.end

mapWindowAttributes:
INCBIN "res/map_window_attributes.bin"
.end
