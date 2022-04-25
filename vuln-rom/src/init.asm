include "constants.inc"
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

    ; Set the gamemode to the map by default (if the map is valid)
    ; TODO: Uncomment this when done working with the menu
    ;ld a, GAMESTATE_MAP
    ld a, GAMESTATE_MENU

    ld [gameState], a

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

    ; Since we have a new save, set the game state to prompt for name and flag
    ld a, GAMESTATE_MENU
    ld [gameState], a

.saveIsValid

    ; Disable SRAM
    ld a, CART_SRAM_DISABLE
    ld [rRAMG], a

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
    ; Check if we really are running on an SGB
    call CheckSGB
    ret nc

    ; Send magic packets to start transfer
    di
    call PrepareSnesTransfer
    ei

    ; Copy the 128 tiles of the frame to SNES RAM
    ld a, 1
    ld [copyingSGBTileData], a
    ld hl, ChrTrnPacket
    ld de, SGBBorderGraphics
    call CopyGfxToSnes

    ; Copy the frame map to SNES RAM
    xor a
    ld [copyingSGBTileData], a
    ld hl, PctTrnPacket
    ld de, BorderPalettes
    call CopyGfxToSnes

    ; Copy the custom game palettes to SNES RAM
    xor a
    ld [copyingSGBTileData], a
    ld hl, PalTrnPacket
    ld de, SGBSuperPalettes
    call  CopyGfxToSnes

    ; Reset VRAM
    ld a, 0
    ld hl, _VRAM
    ld bc, $2000
    call MemSet

    ld hl, MaskEnCancelPacket
    call SendPackets

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

    ; GDMA the sprite tile data
    ld a, HIGH(spriteTileData)
    ld [rHDMA1], a
    ld a, LOW(spriteTileData)
    ld [rHDMA2], a

    ld a, HIGH(_VRAM8000)
    ld [rHDMA3], a
    ld a, LOW(_VRAM8000)
    ld [rHDMA4], a

    ; Start GDMA
    ld a, (((spriteTileData.end - spriteTileData) >> 4) - 1) | HDMA5F_MODE_GP
    ld [rHDMA5], a

    ; GDMA the background tile data
    ld a, HIGH(backgroundTileData)
    ld [rHDMA1], a
    ld a, LOW(backgroundTileData)
    ld [rHDMA2], a

    ld a, HIGH(_VRAM8800)
    ld [rHDMA3], a
    ld a, LOW(_VRAM8800)
    ld [rHDMA4], a

    ; Start GDMA
    ld a, (((backgroundTileData.end - backgroundTileData) >> 4) - 1) | HDMA5F_MODE_GP
    ld [rHDMA5], a

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

    ret

InitDmg::
    ; Init a basic 11100100 palette everywhere for now
    ld a, $e4
    ld [rBGP], a
    ld [rOBP0], a
    ld [rOBP1], a

    ; Copy the sprite tile data
    ld de, spriteTileData
    ld hl, _VRAM8000
    ld bc, spriteTileData.end - spriteTileData
    call MemCpy

    ; Copy the ascii tile data
    ld de, backgroundTileData
    ld hl, _VRAM8800
    ld bc, backgroundTileData.end - backgroundTileData
    call MemCpy

    ; Copy the ascii tile data
    ld de, asciiTileData
    ld hl, _VRAM9000
    ld bc, asciiTileData.end - asciiTileData
    call MemCpy

    ret

; Initialize SNES transfer by sending Freeze and some magic packets
PrepareSnesTransfer::
    ld hl, MaskEnFreezePacket
    call SendPackets
    ld hl, DataSnd0
    call SendPackets
    ld hl, DataSnd1
    call SendPackets
    ld hl, DataSnd2
    call SendPackets
    ld hl, DataSnd3
    call SendPackets
    ld hl, DataSnd4
    call SendPackets
    ld hl, DataSnd5
    call SendPackets
    ld hl, DataSnd6
    call SendPackets
    ld hl, DataSnd7
    call SendPackets
    ret

; Indicate whether the game is running on an SGB.
; @return Carry flag if true
CheckSGB::
    ld hl, MltReq2Packet
    di
    call SendPackets
    ei

    call Wait7000
    ldh a, [rP1]
    and $3
    cp $3
    jr nz, .isSGB

    ld a, $20
    ldh [rP1], a
    ldh a, [rP1]
    ldh a, [rP1]
    call Wait7000
    call Wait7000

    ld a, $30
    ldh [rP1], a
    call Wait7000
    call Wait7000

    ld a, $10
    ldh [rP1], a
    ldh a, [rP1]
    ldh a, [rP1]
    ldh a, [rP1]
    ldh a, [rP1]
    ldh a, [rP1]
    ldh a, [rP1]
    call Wait7000
    call Wait7000

    ld a, $30
    ldh [rP1], a
    ldh a, [rP1]
    ldh a, [rP1]
    ldh a, [rP1]
    call Wait7000
    call Wait7000

    ldh a, [rP1]
    and $3
    cp $3
    jr nz, .isSGB

    call SendMltReq1Packet
    and a
    ret
.isSGB
    call SendMltReq1Packet
    scf
    ret

SendMltReq1Packet:
    ld hl, MltReq1Packet
    call SendPackets
    jp Wait7000

Wait7000:
    ; Each loop takes 9 cycles so this routine actually waits 63000 cycles.
    ld de, 7000
.loop
    nop
    nop
    nop
    dec de
    ld a, d
    or e
    jr nz, .loop
    ret

; Copy graphics data to the SNES
; @param de The graphics data
; @param hl The packet to send
CopyGfxToSnes::
    di
    push hl

    ; Disable LCD during transfer
    ld a, 0
    ld [rLCDC], a
    
    ; Transfer background palette value
    ld a, $e4
    ldh [rBGP], a
    ld hl, _VRAM8800

    ld a, [copyingSGBTileData]
    and a
    jr z, .notCopyingTileData
    call CopySGBBorderTiles
    jr .next

.notCopyingTileData
    ; Copy 4K data from VRAM to SNES
    ld bc, $1000
    call MemCpy

.next
    ; Copy visible background to SNES
    ld hl, _SCRN0
    ld de, $c ; Background additional width
    ld a, $80 ; VRAM address of the first tile
    ld c, $d ; Nb rows

.loop
    ld b, $14 ; Visible background width

.innerLoop
    ld [hli], a ; Tile set
    inc a
    dec b
    jr nz, .innerLoop
    add hl, de ; Next visible background
    dec c
    jr nz, .loop

    ; Turn on LCD to start transfer
    ld a, LCDC_DEFAULT
    ldh [rLCDC], a

    ; Send packet
    pop hl
    call SendPackets

    ; Restore background palette
    xor a
    ldh [rBGP], a
    ei
    ret

; SGB tile data is stored in a 4BPP planar format.
; Each tile is 32 bytes. The first 16 bytes contain bit planes 1 and 2, while
; the second 16 bytes contain bit planes 3 and 4.
; This function converts 2BPP planar data into this format by mapping
; 2BPP colors 0-3 to 4BPP colors 0-3. 4BPP colors 4-15 are not used.
; @param de Graphics data
; @param hl Destination
CopySGBBorderTiles::
    ld b, 128
.tileLoop
; Copy bit planes 1 and 2 of the tile data.
    ld c, 16
.copyLoop
    ld a, [de]
    ld [hli], a
    inc de
    dec c
    jr nz, .copyLoop

; Zero bit planes 3 and 4.
    ld c, 16
    xor a
.zeroLoop
    ld [hli], a
    dec c
    jr nz, .zeroLoop

    dec b
    jr nz, .tileLoop
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
; Defaults to a greyscale palette
.bg0
DW $FFFF, $5294, $294a, $0000
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
; Defaults to a greyscale palette
.obj0
DW $FFFF, $5294, $294a, $0000
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
spriteTileData::
INCBIN "res/sprite_tiles.bin"
.end

backgroundTileData::
INCBIN "res/background_tiles.bin"
.end

asciiTileData::
INCBIN "res/ascii_tiles.bin"
.end

INCLUDE "sgb_packets.inc"
INCLUDE "sgb_border.inc"

SECTION "OAM DMA Hram", HRAM
OamDma::
    ds oamDmaROM.end - oamDmaROM
