#[test]
fn cgb_acid2() {
    let cgb_acid2_image = image::load_from_memory_with_format(
        include_bytes!("./cgb-acid2.png"),
        image::ImageFormat::Png,
    )
    .expect("invalid test image file!");
    let cgb_acid2_image = cgb_acid2_image.to_rgba8().to_vec();

    let cgb_acid2_rom = include_bytes!("./cgb-acid2.gbc");

    let mut emulator = gband::Emulator::new(cgb_acid2_rom, None).expect("Invalid Rom!");

    // Skip a few frames
    for _ in 0..16 {
        loop {
            if let Some(_) = emulator.clock() {
                break;
            }
        }
    }

    let frame = loop {
        if let Some(f) = emulator.clock() {
            break f;
        }
    };

    assert_eq!(frame.as_slice(), &cgb_acid2_image);
}
