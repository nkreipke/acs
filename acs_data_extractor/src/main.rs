use std::fs::{create_dir, File};
use std::io::Write;
use std::path::Path;
use std::process::exit;
use acs::AcsFile;

fn main() {
    let args = std::env::args().collect::<Vec<_>>();
    if args.len() != 3 {
        eprintln!("usage: {} <acs_path> <export_path>", args[0]);
        exit(1);
    }

    let path = &args[1];
    let export_path = &args[2];

    let acs = AcsFile::open_path(path).unwrap();

    println!("ACS file opened: {:?}", acs);

    ensure_dir(export_path);

    for animation in acs.animations() {
        let animation = animation.unwrap();

        println!("extracting {:?}", animation);

        let animation_path = Path::new(export_path).join(animation.name().to_string());
        ensure_dir(&animation_path);

        let mut frame_ms = 0;
        for frame in animation.frames().unwrap() {
            let mut image_i = 0;

            for image in frame.images().unwrap() {
                let image_path = animation_path.join(format!("{frame_ms:06}-{image_i:02}.png"));

                let image = acs.image(image.image_index()).unwrap();
                let (width, height) = image.size();

                let mut rgbdata = vec![];
                image.read_rgba(&mut rgbdata);

                let out = File::create(image_path).unwrap();

                let mut encoder = png::Encoder::new(out, width as u32, height as u32);
                encoder.set_color(png::ColorType::Rgba);
                encoder.set_depth(png::BitDepth::Eight);

                let mut writer = encoder.write_header().unwrap();
                writer.write_image_data(&rgbdata).unwrap();

                image_i += 1;
            }

            if let Some(audio) = frame.audio_index() {
                let audio_path = animation_path.join(format!("{frame_ms:06}.wav"));

                let mut audio_data = vec![];
                acs.audio(audio, &mut audio_data).unwrap();

                let mut out = File::create(audio_path).unwrap();
                out.write_all(&audio_data).unwrap();
            }

            frame_ms += frame.duration().as_millis() as u64;
        }
    }
}

fn ensure_dir(path: impl AsRef<Path>) {
    let p = path.as_ref();
    if !p.exists() {
        create_dir(p).unwrap();
    }
}