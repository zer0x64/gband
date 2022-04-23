include "hardware.inc"

; Those are the flags to set LCDC to
LCDC_DEFAULT = LCDCF_BGON | LCDCF_OBJON | LCDCF_OBJ8 | LCDCF_BG9800 | LCDCF_BG8800 | LCDCF_WINOFF | LCDCF_WIN9C00 | LCDCF_ON

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

    ; Start the game loop
    ld a, BANK(GameLoop)    ;   Load in the bank of the init code
    ld [rROMB0], a

    ld a, LCDC_DEFAULT      ; Enable PPU
    ld [rLCDC], a
    
    ; Enable interrupts
    ei

    jp GameLoop
