use std::io::{Write, stdout};

use crossterm::style::{Color, Print, Stylize};

struct Colour {
    r: u8,
    g: u8,
    b: u8,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let palette = std::env::args()
        .nth(1)
        .expect("Expected palette link as argument 1");
    let size = std::env::args()
        .nth(2)
        .map(|a| a.parse())
        .unwrap_or(Ok(8))?;

    let base = if palette.starts_with("https://lospec.com") {
        palette
    } else if palette.starts_with("lospec.com") {
        format!("https://{palette}")
    } else {
        format!("https://lospec.com/palette-list/{palette}")
    };

    let url = format!("{base}.csv");

    let response = reqwest::blocking::get(url)?;
    let csv = response.text()?;
    let items = csv.split(',').collect::<Vec<_>>();
    let [name, author, colours @ ..] = items.as_slice() else {
        panic!("Invalid API response, expected 'palette_name,author,colours..'");
    };

    println!("{name} by {author}");
    let ideal_side_length = (colours.len() as f64).sqrt();
    let side_length = ideal_side_length.ceil() as u32;

    let mut colours_2d = vec![vec![]];
    for colour in colours {
        let mut current = colours_2d.last_mut().unwrap();
        if current.len() >= side_length as usize {
            colours_2d.push(vec![]);
            current = colours_2d.last_mut().unwrap();
        }
        let r = u8::from_str_radix(&colour[0..2], 16)?;
        let g = u8::from_str_radix(&colour[2..4], 16)?;
        let b = u8::from_str_radix(&colour[4..6], 16)?;

        current.push(Colour { r, g, b });
    }

    let mut imgbuf = image::ImageBuffer::new(side_length * size, side_length * size);

    for (x, y, pixel) in imgbuf.enumerate_pixels_mut() {
        let i = (y / size) as usize;
        let j = (x / size) as usize;
        let colour = colours_2d.get(i).and_then(|a| a.get(j));
        *pixel = if let Some(colour) = colour {
            image::Rgba([colour.r, colour.g, colour.b, 255])
        } else {
            image::Rgba([0, 0, 0, 0])
        }
    }

    imgbuf.save("palette.png")?;

    for line in colours_2d {
        for colour in line {
            let styled = "  ".on(Color::Rgb {
                r: colour.r,
                g: colour.g,
                b: colour.b,
            });
            crossterm::queue!(stdout(), Print(styled))?;
        }
        crossterm::queue!(stdout(), Print('\n'))?;
        stdout().flush()?;
    }

    Ok(())
}
