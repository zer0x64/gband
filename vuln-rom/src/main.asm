SECTION "Game Loop", ROMX
RunGame::
    ld a, 0
    ld [testVariable], a

.loop:
    ; Increment a test variable
    ld a, [testVariable]
    inc a
    ld [testVariable], a

    ; Lock so we wait for the frame to end
    ld a, 1
    ld [waitForFrame], a;
.waitForFrame
    ; Wait until waitForFrame = 0, which is set by the VBlank handler
    ld a, [waitForFrame]
    jr nz, .waitForFrame
    jr .loop
