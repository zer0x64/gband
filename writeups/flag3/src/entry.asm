include "constants.inc"

SECTION "Entry point", ROM0

EntryPoint::
	ld [isCgb], a;		Save A register for CGB value
	ld a, c;
    ld [isSgb], a;      Save C register for SGB value

    ; Turn off LCD during init;
    ld a, 0;
    ld [rLCDC], a ; Completely turn LCD off during init

    ; Setup the stack
    ld sp, wStackBottom

    ld a, BANK(Init)    ;   Load in the bank of the init code
    ld [rROMB0], a

    call Init;

.loop
    ld a, [gameState]
    cp GAMESTATE_MENU
    jr z, .menu
    cp GAMESTATE_INPUT_MENU
    jr z, .inputMenu
    cp GAMESTATE_MAP
    jr z, .map
    cp GAMESTATE_SERIAL
    jr .serial
.menu
    ld a, BANK(RunMenu)    ;   Load in the bank
    ld [rROMB0], a

    call RunMenu
    jr .loop
.inputMenu
    ld a, BANK(RunInputMenu)    ;   Load in the bank
    ld [rROMB0], a

    call RunInputMenu
    jr .loop
.map
    ld a, BANK(RunGame)    ;   Load in the bank
    ld [rROMB0], a

    call RunGame
    jr .loop
.serial
    ld a, BANK(RunSerialMode)    ;   Load in the bank
    ld [rROMB0], a

    call RunSerialMode
    jr .loop
