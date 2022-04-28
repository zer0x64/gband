STACK_SIZE = $40

SECTION "WRAM", WRAM0
localVariables::
    DS $20         ; Reserve space for local variables inside functions
.end::
playerNameLengthRam::
    DB
playerNameRam::
    DS $8         ; Space where the name is stored
.end::
flagLengthRam::
    DB
flagRam::
    DS $10         ; Space where the flag is stored. Note that this is after the local variables so te buffer overflows it and the CTF players nmeeds to fetch it from SRAM
.end::
isCgb::
    DB
isSgb::
    DB
; Used to tell the game in which state it is.
gameState::        
    DB
waitForFrame::
    DB
oldBankNumber::
    DB              ; Used to store bank number to restore it. Useful when needing to jump to ROM0 to access another bank
; Used to stored the joypad state
joypadDpad::
    DB
joypadButtons::
    DB
joypadDpadOld::
    DB
joypadButtonsOld::
    DB
; From here forward, we can declare state-specific variables and they can overlap
copyingSGBTileDataState::
menuCursorPosition::
characterPositionX::
    DB
characterPositionY::
menuState::
    DB
animationCycleTimer::
    DB
characterDirection::
menuInputLength::
    DB
mapState::
menuInput::
    DS $20
wStack::
	ds STACK_SIZE   ; Define a stack here. I make sure it's after "localVariables" so a buffer overflow can overwrite a function pointer here
wStackBottom::

SECTION "Shadow Scroll", WRAM0
shadowScrollX::
    DB
shadowScrollY::
    DB

SECTION UNION "Shadow OAM", WRAM0, ALIGN[8]
shadowOAM::
    DS $A0
