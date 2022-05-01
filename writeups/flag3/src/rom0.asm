include "hardware.inc"
include "constants.inc"

SECTION "Helpers", ROM0

; Copies a block of memory somewhere else
; @param de Pointer to beginning of block to copy
; @param hl Pointer to where to copy (bytes will be written from there onwards)
; @param bc Amount of bytes to copy (0 causes 65536 bytes to be copied)
; @return de Pointer to byte after last copied one
; @return hl Pointer to byte after last written one
; @return bc 0
; @return a 0
; @return f Z set, C reset
MemCpy::
    ; Increment B if C is non-zero
    dec bc
    inc b
    inc c
.loop
    ld a, [de]
    ld [hli], a
    inc de
    dec c
    jr nz, .loop
    dec b
    jr nz, .loop
    ret

; Compares two memory blocks
; @param de Pointer to beginning of block 1
; @param hl Pointer to beginning of block 2
; @param bc Amount of bytes to copy (0 causes 65536 bytes to be compared)
; @return de Pointer to byte after last copied one
; @return hl Pointer to byte after last written one
; @return bc 0
; @return a = 0 is they are equal
; @return f Z set, C reset
MemCmp::
    ; Increment B if C is non-zero
    dec bc
    inc b
    inc c
.loop
    ld a, [de]
    sub [hl]
    jr nz, .end		; Break if these are not equal
    inc de
    inc hl
    dec c
    jr nz, .loop
    dec b
    jr nz, .loop
.end
    ret

; Fill a memory region with a specific byte
; @param a Byte to copy
; @param hl Pointer to where to copy (bytes will be written from there onwards)
; @param bc Amount of bytes to copy (0 causes 65536 bytes to be copied)
; @return de Pointer to byte after last copied one
; @return hl Pointer to byte after last written one
; @return bc 0
; @return a 0
; @return f Z set, C reset
MemSet::
    ; Increment B if C is non-zero
    dec bc
    inc b
    inc c
.loop
    ld [hli], a
    dec c
    jr nz, .loop
    dec b
    jr nz, .loop
    ret

; Copy data into VRAM. Will use GDMA if available
; @param de Pointer to beginning of block to copy
; @param hl Pointer to where to copy (bytes will be written from there onwards)
; @param bc Amount of bytes to copy (0 causes 65536 bytes to be copied)
; @return de Pointer to byte after last copied one
; @return hl Pointer to byte after last written one
; @return bc 0
; @return a 0
; @return f Z set, C reset
CopyToVRAM::
    ld a, [isCgb]
    cp 1
    jr z, .isCgb
    ; On DMG we memcpy
    call MemCpy
    ret
.isCgb
    ; On CGB we call a DMA
    ld a, d
    ld [rHDMA1], a
    ld a, e
    ld [rHDMA2], a

    ld a, h
    ld [rHDMA3], a
    ld a, l
    ld [rHDMA4], a

    ; Convert length to the DMA format
    ld a, c
    srl a
    srl a
    srl a
    srl a
    ld c, a
    ld a, b
    sla a
    sla a
    sla a
    sla a
    or c
    dec a

    ld [rHDMA5], a
    ret

; Call a method in another bank
; @param a Bank number
; @param hl Address of the function to call
; @param localVariables: It's better here to pass data through the localVariable array because of the indirection
JumpInAnotherBank::
    ld [rROMB0], a
    jp hl

; Read the joypad inputs
; There are no params, output goes to WRAM
ReadJoypad::
    ; Set register to get dpad state
    ld a, P1F_GET_DPAD
    ld [rP1], a

    ; Do a bit of processing now to allow a bit of delay...
    ; Copy to the old values
    ld a, [joypadDpad]
    ld [joypadDpadOld], a

    ld a, [joypadButtons]
    ld [joypadButtonsOld], a

    ; Get DPAD
    ld a, [rP1]
    ld [joypadDpad], a

    ; Set register to get button states
    ld a, P1F_GET_BTN
    ld [rP1], a

    ; Read multiple times for rebound effect
    ld a, [rP1]
    ld a, [rP1]
    ld a, [rP1]
    ld a, [rP1]
    ld a, [rP1]

    ld [joypadButtons], a
    ret

; Bitwise operations to get a bit field of which buttons has been newly pressed
; @param a: old button state
; @param b: new button state
; returns a
GetNewlyPushedButtons::
    ; Basic formula: (OLD ^ NEW) & !NEW
    ; By xoring the two, we get the different bits
    xor b

    ; Invert the new bits
    ld c, a
    ld a, b
    ld b, $FF
    xor b

    ; Mask to only get bits that are pressed
    and c
    ret

; Bitwise operations to get a bit field of which buttons has been newly released
; @param a: old button state
; @param b: new button state
; returns a
GetNewlyReleasedButtons::
    ; Basic formula: (OLD ^ NEW) & NEW
    ; By xoring the two, we get the different bits
    xor b
    and b

    ret

ReturnToOldBank::
    ld a, [oldBankNumber]
    ld [rROMB0], a
    ret

; Handler for the VBLANK
VBlankHandler::
	; Save the current state
	push AF
	push BC
	push DE
	push HL

	; Copy Shadow OAM to OAM
    call OamDma

	; Load in the shadow scroll values
	ld a, [shadowScrollX]
	ld [rSCX], a

	ld a, [shadowScrollY]
	ld [rSCY], a

	; Tell the CPU it can continue the game loop
    ld a, 0
    ld [waitForFrame], a

	; Load the state before the interrupt
	pop HL
	pop DE
	pop BC
	pop AF
    reti

; Handler for the serial
SerialHandler::
    push af
    push bc
    push de
    push hl

    ; Check if connected
    ld a, [serialConnectionState]
    inc a
    jr z, .notConnected

    ; Store received byte and retrieve byte to send
    ldh a, [rSB]
    ld [serialReceiveData], a
    ld a, [serialSendData]
    ldh [rSB], a
    ld a, [serialConnectionState]
    cp SERIAL_CONNECTION_STATE_INTERNAL
    jr z, .done

    ; Not using internal clock, start transfer as external
    ld a, SCF_START
    ldh [rSC], a
    jr .done

.notConnected
    ; Use internal clock if received byte says so
    ldh a, [rSB]
    ld [serialReceiveData], a
    ld [serialConnectionState], a
    cp SERIAL_CONNECTION_STATE_INTERNAL
    jr z, .useInternalClock

    ; Otherwise, use external clock
    xor a
    ldh [rSB], a
    ld a, $3
    ldh [rDIV], a
:
    ldh a, [rDIV]
    bit 7, a
    jr nz, :-
    ld a, SCF_START
    ldh [rSC], a
    jr .done

.useInternalClock
    xor a
    ldh [rSB], a

.done
    ld a, 1
    ld [serialReceivedNewData], a

    ; Make "no data" the next byte to send
    ;ld a, SERIAL_NO_DATA_BYTE
    ;ld [serialSendData], a

    pop hl
    pop de
    pop bc
    pop af
    reti
