use num::Complex;
use std::str::FromStr;
use image::{RgbImage, Rgb};
use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() != 6 {
        eprint!("Usage: {}, FILE, PIXELS, UPPERLEFT, LOWERRIGHT MULTI ", args[0]);
        eprintln!("Example: {} mandel.png 1000x750 -1.20, 0.35, -1, 0.20, Multi", args[0]);
        std::process::exit(1);
    }

    let bounds = parse_pair(&args[2], 'x')
        .expect("error parsing image dimensions");
    let upper_left = parse_complex(&args[3])
        .expect("error parsing upper left corner point");
    let lower_right = parse_complex(&args[4])
        .expect("error parsing lower right corner point");
    let multi = parse_multi(&args[5])
        .expect("error parsing multi argument - can only be 'Single' or 'Multi'");

    let mut pixels = RgbImage::new(bounds.0 as u32, bounds.1 as u32);

    match multi {
        true => render_single(&mut pixels, bounds, upper_left, lower_right), // TODO - call render_multi when fixed
        false => render_single(&mut pixels, bounds, upper_left, lower_right)
    }    

    pixels.save(&args[1])
        .expect("error writing png file");
}

fn parse_multi(arg: &str) -> Option<bool> {
    match arg {
        "Multi" => Some(true),
        "Single" => Some(false),
        _ => None
    }
}

#[test]
fn test_parse_multi()
{
    assert_eq!(parse_multi("Single"), Some(false));
    assert_eq!(parse_multi("Multi"), Some(true));
    assert_eq!(parse_multi("FooBar"), None);
    assert_eq!(parse_multi(""), None);

}

/// Try to determine if 'c' is in the Mandelbrot set, using at most 'limit'
/// iterations to decide.
/// 
/// If 'c' is not a member, return 'Some(i)' where 'i' is the number of 
/// iterations it took for 'c' to leave the circle of radius 2 centered on the 
/// origin. If 'c' seems to be a member (more precisely, if we reached the 
/// iteration limit without being able to prove that 'c' is not a member),
/// return None.
fn escape_time(c: Complex<f64>, limit: usize) -> Option<usize> {
    
    let mut z = Complex {re: 0.0, im: 0.0 };
    for i in 0..limit {
        if z.norm_sqr() > 4.0 {
            return Some(i);
        }
        z = z * z + c;
    }

    None
}

/// Parse the string 's' as a coordinate pair, like '"400x600" or '"1.0,0.5"'.
/// 
/// Specifically, 's' should have the form <left><separator><right> where <sep> is
/// the character given by the 'separator' argument, and <left>, and <right> are
/// both strings that can be parsed by 'T::from_str'. 'separator' must be an
/// ASCII character.
/// 
/// if 's' has the proper form, return 'Some<(x,y)>'. If it doesn't parse
/// correctly, return 'None'.
fn parse_pair<T: FromStr>(s: &str, separator:char) -> Option<(T,T)> {
    match s.find(separator) {
        None => None,
        Some(index) => {
            match (T::from_str(&s[..index]), T::from_str(&s[index + 1..])) {
                (Ok(l), Ok(r)) => Some((l,r)),
                _ => None
            }
        }
    }
}

#[test]
fn test_parse_pair() {
    assert_eq!(parse_pair::<i32>("",            ','), None);
    assert_eq!(parse_pair::<i32>("10",          ','), None);
    assert_eq!(parse_pair::<i32>(",10",         ','), None);
    assert_eq!(parse_pair::<i32>("10,20",       ','), Some((10,20)));
    assert_eq!(parse_pair::<i32>("10,20xy",     ','), None);
    assert_eq!(parse_pair::<f64>("0.5x",        ','), None);
    assert_eq!(parse_pair::<f64>("0.5x1.5",     'x'), Some((0.5,1.5)));
}

fn parse_complex(s: &str) -> Option<Complex<f64>> {
    match parse_pair(s, ',') {
        Some((re,im)) => Some(Complex {re, im}),
        None => None    
    }
}

#[test]
fn test_parse_complex() {
    assert_eq!(parse_complex("1.25,-0.0625"), Some(Complex {re: 1.25, im: -0.0625}));
    assert_eq!(parse_complex(",-0.0625"), None);
}

/// Given the row and column of a pixel in the output image, return the
/// corresponding point on the complex plane.
/// 
/// 'bounds' is a pair giving the width and height of the image in pixels.
/// 'pixel' is a (column, row) pair indicating a particular pixel in that image.
/// The 'upper_left' and 'lower_right' parameters are points on the complex
/// plane designating the area our image covers.
fn pixel_to_point(bounds: (usize,usize),
                  pixel:(usize, usize),
                  upper_left: Complex<f64>,
                  lower_right: Complex<f64>) -> Complex<f64>
{
    let (width, height) = (lower_right.re - upper_left.re,
                                     upper_left.im - lower_right.im);
    Complex { 
        re: upper_left.re + pixel.0 as f64 * width / bounds.0 as f64, 
        im: upper_left.im - pixel.1 as f64 * height / bounds.1 as f64 
    }
}

#[test]
fn test_pixel_to_point() {
    assert_eq!(pixel_to_point((100,200), (25,175), 
            Complex{ re: -1.0, im: 1.0}, 
            Complex {re: 1.0, im: -1.0}), 
        Complex { re: -0.5, im:-0.75});
}

/// Render a rectangle of the Mandelbrot set into a buffer of pixels.
/// 
/// The 'bounds' argument gives the width and height fo the buffer. 'pixels',
/// which holds one grayscale pixel per byte. The 'upper_left' and 'lower_right'
/// arguments specify points on the complex plane corresponding to the upper-left
///  and lower-right corners of the pixel buffer
fn render(pixels:&mut RgbImage,
          bounds: (usize, usize),
          upper_left: Complex<f64>,
          lower_right: Complex<f64>)
{
    assert!(pixels.len() == bounds.0 * bounds.1 * 3);

    for row in 0..bounds.1 {
        for column in 0..bounds.0 {
            let point = pixel_to_point(bounds, (column,row), upper_left, lower_right);
            let pixel_value = match escape_time(point,255) {
                None => 0,
                Some(count) => 255 - count as u8
            };
           
            let pixel_color = map_color(pixel_value);
            pixels.put_pixel(column as u32, row as u32, pixel_color);
        }
    }
}

fn map_color(value: u8) -> image::Rgb<u8>
{
    match value {
        0 => Rgb([0,0,0]),
        1..=35 => Rgb([148, 0, 211]),       // Violet
        36..=70 => Rgb([75, 0, 130]),       // Indigo
        71..=105 => Rgb([0, 0, 255]),       // Blue
        106..=140 => Rgb([0, 255, 0]),      // Green
        141..=175 => Rgb([255, 255, 0]),    // Yellow
        176..=210 => Rgb([255, 127, 0]),    // Orange
        211..=254 => Rgb([255, 0, 0]),      // Red
        255 => Rgb([255,255,255])           // White
    }
}

fn render_single(pixels:&mut RgbImage,
    bounds: (usize, usize),
    upper_left: Complex<f64>,
    lower_right: Complex<f64>)
{
    render(pixels, bounds, upper_left, lower_right);
}


