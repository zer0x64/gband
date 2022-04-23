include "hardware.inc"
SECTION FRAGMENT "INIT", ROMX

Init::
    ; We are not waiting for a frame
    ld a, 0
    ld [waitForFrame], a

	; Check and init Sgb frame
    ld a, [isSgb]
	cp $14
	jr z, .runningInSgb

	; Not running in SGB, setting the value to 0
	ld a, 0
	jr .sgbCheckComplete
	
.runningInSgb
	; Running in SGB mode, init frame and set the value to 1
	call InitSgb
	ld a, 1

	.sgbCheckComplete
	ld [isSgb], a

	; Check A for CGB now
	ld a, [isCgb]
	cp $11
	jr z, .runningInCgb

	; Not running in CGB, setting the value to 0 and init DMG
    call InitDmg
	ld a, 0
	jr .cgbCheckComplete
	
.runningInCgb
	; Running in CGB mode, init frame and set the value to 1
	call InitCgb
	ld a, 1

.cgbCheckComplete
	ld [isCgb], a

    ; Validate and create save file
    ; Enable SRAM
    ld a, CART_SRAM_ENABLE
    ld [rRAMG], a

    ; Switch the bank
    ld a, BANK(saveHeader)
    ld [rRAMB], a

    ; Validate the header
    ld de, expectedSramHeader
    ld hl, saveHeader
    ld bc, saveHeader.end - saveHeader
    call MemCmp

    cp a, 0
    jr z, .saveIsValid
    
    ; Save is invalid, reset it
    ; Copy header
    ld de, expectedSramHeader
    ld hl, saveHeader
    ld bc, saveHeader.end - saveHeader
    call MemCpy

    ; Set everything else to 0
    ld a, 0
    ld hl, saveHeader.end
    ld bc, sramEnd - saveHeader.end
    call MemSet

.saveIsValid

    ; Disable SRAM
    ld a, CART_SRAM_DISABLE
    ld a, rRAMG

    ; Set scroll values to 0
    ld a, 0
    ld [rSCY], a
    ld [rSCX], a

    ; Copy OAM DMA routine
    ld de, oamDmaROM
    ld hl, OamDma
    ld bc, oamDmaROM.end - oamDmaROM
    call MemCpy

    ; Fill shadow OAM with 0
    ld a, 0
    ld hl, shadowOAM
    ld bc, $A0
    call MemSet

    ld a, IEF_VBLANK            ; The only interupt we want is the VBLANK
    ld [rIE], a

    ld a, AUDENA_OFF            ; Sound OFF since we don't use it
    ld [rNR52], a
    ret

InitSgb::
    ; TODO: Init SGB frame
    ; Copy the ascii tile data
    ld de, asciiTileData
    ld hl, _VRAM8000
    ld bc, asciiTileData.end - asciiTileData
    call MemCpyTo4bpp

    ; Setup the PPU for transfer
    call FillScreenWithSGBMap

    ld a, 0
    ld hl, localVariables
    ld bc, localVariables.end - localVariables
    call MemSet

    ; Send the ASCII table to the SNES
    ld a, ($13 << 3) | 1        ; CHR_TRAN header
    ld [localVariables], a
    ld a, 0                     ; First bank | Background
    ld [localVariables + 1], a

    ; Send the packet
    ld hl, localVariables
    call SendPackets

    ; Wait 5 frames for transfer
    call WaitFor5Frames

    ; Disable LCD during transfer
    ld a, 0
    ld [rLCDC], a

    ; Copy the border tile map tile data
    ld de, sgbBorderTileMap
    ld hl, _VRAM8000
    ld bc, sgbBorderTileMap.end - sgbBorderTileMap
    call MemCpy

    ; Restart PPU
    call SetupSGBLCDC

    ; Send the border tilemap to the SNES
    ld a, ($14 << 3) | 1        ; PCT_TRAN header
    ld [localVariables], a
    ld a, 0                     ; First bank | Background
    ld [localVariables + 1], a

    ; Send the packet
    ld hl, localVariables
    call SendPackets

    ; Wait 5 frames for transfer
    call WaitFor5Frames

    ; Shut down the PPU
    ld a, 0
    ld [rLCDC], a

    ret
InitCgb::
    ; Init the palettes
    ; Background Palette
    ld a, $80;      ; Autoincrement, start at 0
    ld [rBCPS], a

    ld de, cgbBackgroundPalette                          ; Array to copy
    ld c, cgbBackgroundPalette.end - cgbBackgroundPalette ; Loop counter

    ; Init VRAM bank
    ld a, 0
    ld [rVBK], a

.background_copy_loop
    ld a, [de]
    ld [rBCPD], a; Write byte to palette data
    inc de
    dec c
    jr nz, .background_copy_loop
    
    ; Object Palette
    ld a, $80;      ; Autoincrement, start at 0
    ld [rOCPS], a

    ld de, cgbObjectPalette                          ; Array to copy
    ld c, cgbObjectPalette.end - cgbObjectPalette ; Loop counter

.object_copy_loop
    ld a, [de]
    ld [rOCPD], a; Write byte to palette data
    inc de
    dec c
    jr nz, .object_copy_loop

    ; GDMA the ASCII tile data
    ld a, HIGH(asciiTileData)
    ld [rHDMA1], a
    ld a, LOW(asciiTileData)
    ld [rHDMA2], a

    ld a, HIGH(_VRAM9000)
    ld [rHDMA3], a
    ld a, LOW(_VRAM9000)
    ld [rHDMA4], a

    ; Start GDMA
    ld a, (((asciiTileData.end - asciiTileData) >> 4) - 1) | HDMA5F_MODE_GP
    ld [rHDMA5], a

    ; GDMA the tile map
    ld a, HIGH(menuTileMap)
    ld [rHDMA1], a
    ld a, LOW(menuTileMap)
    ld [rHDMA2], a

    ld a, HIGH(_SCRN0)
    ld [rHDMA3], a
    ld a, LOW(_SCRN0)
    ld [rHDMA4], a

    ; Start GDMA
    ld a, (((menuTileMap.end - menuTileMap) >> 4) - 1) | HDMA5F_MODE_GP
    ld [rHDMA5], a

    ; GDMA the attribute map
    ; Change VRAM bank
    ld a, 1
    ld [rVBK], a

    ld a, HIGH(menuAttributes)
    ld [rHDMA1], a
    ld a, LOW(menuAttributes)
    ld [rHDMA2], a

    ld a, HIGH(_SCRN0)
    ld [rHDMA3], a
    ld a, LOW(_SCRN0)
    ld [rHDMA4], a

    ; Start GDMA
    ld a, (((menuAttributes.end - menuAttributes) >> 4) - 1) | HDMA5F_MODE_GP
    ld [rHDMA5], a

    ; Reset VRAM bank
    ld a, 0
    ld [rVBK], a

    ret

InitDmg::
    ; Init a basic 11100100 palette everywhere for now
    ld a, $e4
    ld [rBGP], a
    ld [rOBP0], a
    ld [rOBP1], a

    ; Copy the ascii tile data
    ld de, asciiTileData
    ld hl, _VRAM9000
    ld bc, asciiTileData.end - asciiTileData
    call MemCpy

    ; Copy the tile map
    ld de, menuTileMap
    ld hl, _SCRN0
    ld bc, menuTileMap.end - menuTileMap
    call MemCpy

    ret

WaitFor5Frames::
    ld b, 10
.waitForVblankOn
    ld a, [rSTAT]
    and %11
    cp 1
    jr z, .waitForVblankOn
.waitForVblankOff
    ld a, [rSTAT]
    and %11
    cp 1
    jr nz, .waitForVblankOff
    dec b
    jr nz, .waitForVblankOn
    ret

MemCpyTo4bpp::
	; Increment B if C is non-zero
	dec bc
	inc b
	inc c
.loop
	ld a, [de]
	ld [hli], a
	ld [hli], a
	inc de
	dec c
	jr nz, .loop
	dec b
	jr nz, .loop
	ret

oamDmaROM::
    ld a, HIGH(shadowOAM)
    ldh [rDMA], a
    ld a, 40; Wait for 160 cycles
.wait
    dec a
    jr nz, .wait
    ret
.end

; TODO: Put the actual palettes
cgbBackgroundPalette::
.bg0
DW $0000, $1111, $2222, $3333
.bg1
DW $4444, $5555, $6666, $7777
.bg2
DW $8888, $9999, $AAAA, $BBBB
.bg3
DW $CCCC, $DDDD, $EEEE, $FFFF
.bg4
DW $0000, $1010, $2323, $3232
.bg5
DW $4545, $5454, $6767, $7676
.bg6
DW $8989, $9898, $ABAB, $BABA
.bg7 
DW $CDCD, $DCDC, $EFEF, $FEFE
.end

cgbObjectPalette::
.obj0
DW $0000, $1111, $2222, $3333
.obj1
DW $4444, $5555, $6666, $7777
.obj2
DW $8888, $9999, $AAAA, $BBBB
.obj3
DW $CCCC, $DDDD, $EEEE, $FFFF
.obj4
DW $0000, $1010, $2323, $3232
.obj5
DW $4545, $5454, $6767, $7676
.obj6
DW $8989, $9898, $ABAB, $BABA
.obj7 
DW $CDCD, $DCDC, $EFEF, $FEFE
.end

expectedSramHeader::
db "Super Myco Boi!!"
.end

SECTION FRAGMENT "INIT", ROMX, ALIGN[8]
asciiTileData::
INCBIN "res/ascii_tiles.bin"
.end

menuTileMap::
INCBIN "res/menu_tilemap.bin"
.end

sgbBorderTileMap::
INCBIN "res/sgb_border_tilemap.bin"
.end

menuAttributes::
INCBIN "res/menu_attributes.bin"
.end

SECTION "OAM DMA Hram", HRAM
OamDma::
    ds oamDmaROM.end - oamDmaROM
