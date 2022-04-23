STACK_SIZE = $40

SECTION "WRAM", WRAM0
localVariables::
    DS $20         ; Reserve space for local variables inside functions
.end::
flagRam::
    DS $20         ; Space where the flag is stored. Note that this is after the local variables so te buffer overflows it and the CTF players nmeeds to fetch it from SRAM
.end::
isCgb::
    DB
isSgb::
    DB
waitForFrame::
    DB
oldBankNumber::
    DB              ; Used to store bank number to restore it. Useful when needing to jump to ROM0 to access another bank
testVariable::
    DB
wStack::
	ds STACK_SIZE   ; Define a stack here. I make sure it's after "localVariables" so a buffer overflow can overwrite a function pointer here
wStackBottom::

SECTION UNION "Shadow OAM", WRAM0, ALIGN[8]
shadowOAM::
    DS $A0
