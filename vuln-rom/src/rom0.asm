include "hardware.inc"

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

; Call a method in another bank
; @param a Bank number
; @param hl Address of the function to call
; @param localVariables: It's better here to pass data through the localVariable array because of the indirection
JumpInAnotherBank::
	ld [rROMB0], a
	jp hl

ReturnToOldBank::
	ld a, [oldBankNumber]
	ld [rROMB0], a
	ret

; Handler for the VBLANK
VBlankHandler::
    call OamDma     ; Copy Shadow OAM to OAM
    ld a, 0         ; Tell the CPU it can continue the game loop
    ld [waitForFrame], a
    reti
