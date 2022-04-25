ATTR_BLK: MACRO
    ; This is a command macro.
    ; Use ATTR_BLK_DATA for data sets.
    db ($4 << 3) + ((\1 * 6) / 16 + 1)
    db \1
ENDM

ATTR_BLK_DATA: MACRO
    db \1 ; which regions are affected
    db \2 + (\3 << 2) + (\4 << 4) ; palette for each region
    db \5, \6, \7, \8 ; x1, y1, x2, y2
ENDM

PAL_SET: MACRO
    db ($a << 3) + 1
    dw \1, \2, \3, \4
    ds 7, 0
ENDM

PAL_TRN: MACRO
    db ($b << 3) + 1
    ds 15, 0
ENDM

MLT_REQ: MACRO
    db ($11 << 3) + 1
    db \1 - 1
    ds 14, 0
ENDM

CHR_TRN: MACRO
    db ($13 << 3) + 1
    db \1 + (\2 << 1)
    ds 14, 0
ENDM

PCT_TRN: MACRO
    db ($14 << 3) + 1
    ds 15, 0
ENDM

MASK_EN: MACRO
    db ($17 << 3) + 1
    db \1
    ds 14, 0
ENDM

DATA_SND: MACRO
    db ($f << 3) + 1
    dw \1 ; address
    db \2 ; bank
    db \3 ; length (1-11)
ENDM

PalTrnPacket:  PAL_TRN
MltReq1Packet: MLT_REQ 1
MltReq2Packet: MLT_REQ 2
ChrTrnPacket:  CHR_TRN 0, 0
PctTrnPacket:  PCT_TRN

MaskEnFreezePacket: MASK_EN 1
MaskEnCancelPacket: MASK_EN 0

PalPacket_Empty: PAL_SET 0, 0, 0, 0

; Magic packets for SNES transfer initialization

dbw: MACRO
    db \1
    dw \2
ENDM

DataSnd0:
    DATA_SND $85d, $0, 11
    db  $8C                 ; cpx #$8c (2)
    db  $D0, $F4            ; bne -$0c
    db  $60                 ; rts
    ds  7, 0

DataSnd1:
    DATA_SND $852, $0, 11
    db  $A9, $E7            ; lda #$e7
    db  $9F, $01, $C0, $7E  ; sta $7ec001, x
    db  $E8                 ; inx
    db  $E8                 ; inx
    db  $E8                 ; inx
    db  $E8                 ; inx
    db  $E0                 ; cpx #$8c (1)

DataSnd2:
    DATA_SND $847, $0, 11
    db  $C4                 ; cmp #$c4 (2)
    db  $D0, $16            ; bne +$16
    db  $A5                 ; lda dp
    db  $CB                 ; wai
    db  $C9, $05            ; cmp #$05
    db  $D0, $10            ; bne +$10
    db  $A2, $28            ; ldx #$28

DataSnd3:
    DATA_SND $83c, $0, 11
    db  $F0, $12            ; beq +$12
    db  $A5                 ; lda dp
    db  $C9, $C9            ; cmp #$c9
    db  $C8                 ; iny
    db  $D0, $1C            ; bne +$1c
    db  $A5                 ; lda dp
    db  $CA                 ; dex
    db  $C9                 ; cmp #$c4 (1)

DataSnd4:
    DATA_SND $831, $0, 11
    dbw $0C, $CAA5          ; tsb $caa5
    db  $C9, $7E            ; cmp #$7e
    db  $D0, $06            ; bne +$06
    db  $A5                 ; lda dp
    db  $CB                 ; wai
    db  $C9, $7E            ; cmp #$7e

DataSnd5:
    DATA_SND $826, $0, 11
    db  $39                 ; bne +$39 (2)
    dbw $CD, $C48           ; cmp $c48
    db  $D0, $34            ; bne +$34
    db  $A5                 ; lda dp
    db  $C9, $C9            ; cmp #$c9
    db  $80, $D0            ; bra -$30

DataSnd6:
    DATA_SND $81b, $0, 11
    db  $EA                 ; nop
    db  $EA                 ; nop
    db  $EA                 ; nop
    db  $EA                 ; nop
    db  $EA                 ; nop
                            ; $820:
    db  $A9,$01             ; lda #01
    dbw $CD,$C4F            ; cmp $c4f
    db  $D0                 ; bne +$39 (1)

DataSnd7:
    DATA_SND $810, $0, 11
    dbw $4C, $820           ; jmp $820
    db  $EA                 ; nop
    db  $EA                 ; nop
    db  $EA                 ; nop
    db  $EA                 ; nop
    db  $EA                 ; nop
    db  $60                 ; rts
    db  $EA                 ; nop
    db  $EA                 ; nop
