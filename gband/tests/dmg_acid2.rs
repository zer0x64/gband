#[test]
fn dmg_acid2() {
    let dmg_acid2_image = image::load_from_memory_with_format(
        include_bytes!("./dmg-acid2.png"),
        image::ImageFormat::Png,
    )
    .expect("invalid test image file!");
    let dmg_acid2_image = dmg_acid2_image.to_rgba8().to_vec();

    let dmg_acid2_rom = include_bytes!("./dmg-acid2.gb");

    let mut emulator = gband::Emulator::new(dmg_acid2_rom, None).expect("Invalid Rom!");

    // Skip a few frames
    for _ in 0..10 {
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

    assert_eq!(frame.as_slice(), &dmg_acid2_image);
}
