; ------------------------------
; This simple test rom will send a counter through
; the serial port
;
; One machine will be initiated as P1 and
; the other as P2 using Start and Select
;
; P1 will have an even counter, and P2
; an odd counter
;
; The 2 players send each other their counter
; increment, and send back the counter they received
; ------------------------------
include "hardware.inc"

; ------------------------------
; RAM variables
; ------------------------------
def PAD equ _RAM ; Current state of the joypad
def LOCAL_COUNTER equ _RAM+1 ; Local player counter
def RECEIVED_COUNTER equ _RAM+2 ; Received counter for debugging
def COMPARE_RESULT equ _RAM+3 ; Comparison result, if zero then it's equal

; ------------------------------
; Main executable code
; ------------------------------
section "Main", rom0[$0100]
	nop
	jp main

	ds $150 - @, 0 ; Make room for the header

main:
	di
	ld sp, $ffff

	ld hl, rSB

.wait_input:
	call read_joypad

	ld a, [PAD]	
	and PADF_START
	jr nz, .p1

	ld a, [PAD]
	and PADF_SELECT
	jr nz, .p2

	call delay
	jr .wait_input

.p1:
	; Init the counter
	ld a, 0
	ld [LOCAL_COUNTER], a

:
	; Send P1 counter
	ld a, [LOCAL_COUNTER]
	call sio_master_transfer
	call sio_wait_transfer

	; Read P2 counter, increment and send back
	ld a, [hl]
	add 2
	call sio_slave_transfer
	call sio_wait_transfer

	; Get returned counter
	ld a, [hl]
	ld [RECEIVED_COUNTER], a
	ld b, a

	; Increment P1 Counter
	ld a, [LOCAL_COUNTER]
	add 2
	ld [LOCAL_COUNTER], a
	
	; Compare for fun (not much that can be done without display)
	sub b
	ld [COMPARE_RESULT], a

	; Loop
	ld bc, 2
	ld de, 40000
	call big_delay
	jr :-

.p2:
	; Init the counter
	ld a, 1
	ld [LOCAL_COUNTER], a

:
	; Receive P1 counter, add and send back
	ld a, [LOCAL_COUNTER]
	call sio_slave_transfer
	call sio_wait_transfer
	ld a, [hl]
	add 2
	call sio_master_transfer
	call sio_wait_transfer

	; Increment P2 Counter
	ld a, [LOCAL_COUNTER]
	add 2
	ld [LOCAL_COUNTER], a

	; Retrieve counter returned by P1
	ld a, [hl]
	ld [RECEIVED_COUNTER], a
	ld b, a

	; Compare for fun (not much that can be done without display)
	sub b
	ld [COMPARE_RESULT], a

	; Loop
	ld bc, 2
	ld de, 40000
	call big_delay
	jr :-


; ------------------------------
; Get pressed buttons on the joypad
;
; Stores the result in `PAD`
; ------------------------------
read_joypad:
	push bc

	ld a, P1F_GET_DPAD
	ld [rP1], a

	; Read the state of the dpad, with bouncing protection
	ld a, [rP1]
	ld a, [rP1]
	ld a, [rP1]
	ld a, [rP1]

	and $0F
	swap a
	ld b, a

	ld a, P1F_GET_BTN
	ld [rP1], a

	; Read the state of the buttons, with bouncing protection
	ld a, [rP1]
	ld a, [rP1]
	ld a, [rP1]
	ld a, [rP1]

	and $0F
	or b

	cpl
	ld [PAD], a

	pop bc

	ret

; ------------------------------
; Wait for 60000+15 cycles (~1.4ms)
; ------------------------------
delay:
	push de
	ld de, 6000
:
	dec de
	ld a, d
	or e
	jr z, :+
	nop
	jr :-
:
	pop de
	ret

; ------------------------------
; Wait for about (de*10) * bc cycles
;
; Inputs:
;   - de: Nb cycles per iteration
;   - bc: Nb iterations
; ------------------------------
big_delay:
.iter:
	dec bc
	ld a, b
	or c
	jr z, .end

.loop:
	dec de
	ld a, d
	or e
	jr z, .iter
	nop
	jr .loop

.end
	ret


;;;
; .loop:
; 	ld a, [SIOTYPE]
; 	or a
; 	jr nz, .start

; 	call read_joypad
; 	ld a, [PAD]
; 	and PADF_START
; 	jp z, .skip

; 	call init_sio_master

; .run:
; 	call read_joypad
; 	ld a, [PAD]
; 	ld [TD], a

; 	ld a, [SIOTYPE]
; 	cp MASTER_MODE
; 	call z, init_sio_master

; .skip:
; 	ld a, [RD]
; 	;???

; .done:
; 	call delay
; 	jr .loop

;; .slave:
;; 	jr .done

;; .master:
;; 	ld a, [PAD]
;; 	ld [rSB], a

;; 	ld a, SCF_START | SCF_SOURCE
;; 	ld [rSC], a
;; 	jr .done



;;;
; ld a, [PAD]
; ld [rSB], a

; Temp write a newline for log
; ld a, 10
; ld [rSB], a

; ld a, SCF_START | SCF_SOURCE
; ld [rSC], a

;; ld a, [PAD]
;; and PADF_SELECT
;; jr nz, .slave
;; ld a, [PAD]
;; and PADF_START
;; jr z, .master
