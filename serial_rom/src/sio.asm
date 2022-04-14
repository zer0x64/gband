section "SIO Code", rom0

include "hardware.inc"

; ------------------------------
; Set buffer and initiate transfer as slave
;
; Inputs:
;	- a: Data to put in the buffer
; ------------------------------
sio_slave_transfer::
	ld [rSB], a
	ld a, SCF_START
	ld [rSC], a
    ret

; ------------------------------
; Set buffer and initiate transfer as master
;
; Inputs:
;	- a: Data to put in the buffer
; ------------------------------
sio_master_transfer::
	ld [rSB], a
	ld a, SCF_START | SCF_SOURCE
	ld [rSC], a
    ret

; ------------------------------
; Busy wait / poll for transfer completion
; ------------------------------
sio_wait_transfer::
.loop
	ld a, [rSC]
	and SCF_START
	jr nz, .loop

	ret
	
