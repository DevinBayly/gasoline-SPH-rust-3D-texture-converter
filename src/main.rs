use byteorder::{BigEndian, LittleEndian, ReadBytesExt};
use image::{DynamicImage,GenericImage, GenericImageView, RgbImage, GrayImage, ImageBuffer, Luma};
use std::fs::read;
use std::io::Cursor;
use std::thread;

#[derive(Debug, Clone, Copy)]
struct Particle {
    mass: f32,
    pos: [f32; 3],
    vel: [f32; 3],
    rho: f32,
    temp: f32,
    eps: f32,
    metals: f32,
    phi: f32,
}

struct Extent {
    min: f32,
    max: f32,
}

impl Extent {
    fn new(starting_value: f32) -> Self {
        Self {
            min: starting_value,
            max: starting_value,
        }
    }
    fn comp(&mut self, other: f32) {
        if other < self.min {
            self.min = other;
        } else if other > self.max {
            self.max = other;
        }
    }
    fn norm(&mut self, other: f32) -> f32 {
        (other - self.min) / (self.max - self.min)
    }
}

// want to create a way to see the actual positions
// still have
fn main() {
    let mut buff = read("./agora.000010").unwrap();
    let mut rdr = Cursor::new(buff);

    let time = rdr.read_f64::<LittleEndian>().unwrap();
    let nbodies = rdr.read_i32::<LittleEndian>().unwrap();
    let ndim = rdr.read_i32::<LittleEndian>().unwrap();
    let nsph = rdr.read_i32::<LittleEndian>().unwrap();
    let ndark = rdr.read_i32::<LittleEndian>().unwrap();
    let nstar = rdr.read_i32::<LittleEndian>().unwrap();
    let pad = rdr.read_i32::<LittleEndian>().unwrap();
    // first gas particle
    let mut sphs = vec![];
    for i in 0..nsph {
        sphs.push(Particle {
            mass: rdr.read_f32::<LittleEndian>().unwrap(),
            pos: [
                rdr.read_f32::<LittleEndian>().unwrap(),
                rdr.read_f32::<LittleEndian>().unwrap(),
                rdr.read_f32::<LittleEndian>().unwrap(),
            ],
            vel: [
                rdr.read_f32::<LittleEndian>().unwrap(),
                rdr.read_f32::<LittleEndian>().unwrap(),
                rdr.read_f32::<LittleEndian>().unwrap(),
            ],
            rho: rdr.read_f32::<LittleEndian>().unwrap(),
            temp: rdr.read_f32::<LittleEndian>().unwrap(),
            eps: rdr.read_f32::<LittleEndian>().unwrap(),
            metals: rdr.read_f32::<LittleEndian>().unwrap(),
            phi: rdr.read_f32::<LittleEndian>().unwrap(),
        });
    }
    println!("length of sphs {:?}", sphs.len());
    let mut x_extent = Extent::new(sphs[0].pos[0]);
    let mut y_extent = Extent::new(sphs[0].pos[1]);
    let mut z_extent = Extent::new(sphs[0].pos[2]);
    for particle in sphs.iter() {
        x_extent.comp(particle.pos[0]);
        y_extent.comp(particle.pos[1]);
        z_extent.comp(particle.pos[2]);
    }
    // go through and normalize the sphs
    let mut normalized_sphs = sphs.clone();
    for particle in normalized_sphs.iter_mut() {
        particle.pos[0] = x_extent.norm(particle.pos[0]);
        particle.pos[1] = y_extent.norm(particle.pos[1]);
        particle.pos[2] = z_extent.norm(particle.pos[2]);
    }
    // create an image

    let side_len = (nsph as f32).sqrt() as u32;
    let mut img = DynamicImage::new_rgb16(side_len,side_len).to_rgb16();

    // read the individual particles into the image
    let mut end = false;
    // iterate over the pixels
    let mut i = 0;
    let scalar = 100000.0;
    for (x,y,pixel) in img.enumerate_pixels_mut() {
        match normalized_sphs.get(i) {
            Some(particle) => {
                println!("{:?}",[(particle.pos[0]*scalar) ,(particle.pos[1]*scalar) ,(particle.pos[2]*scalar)]);
                *pixel = image::Rgb([(particle.pos[0]*scalar) as u16,(particle.pos[1]*scalar) as u16,(particle.pos[2]*scalar) as u16]);
            }
            _=> break
        };
        i+=1;
    }
    img.save("test_positions.png").unwrap();

}
fn make_tile_image(
    layer_vector: Vec<Particle>,
    layer_index: u32,
    image_dims: (u32, u32),
    side_len: u32,
) {
    let mut img: GrayImage = ImageBuffer::new(image_dims.0, image_dims.1);

    // go through the normalized_particles and put them on the canvas if we find them by multiplying the xy by the dimensions

    for particle in layer_vector {
        // the -1 is to ensure we don't write at the far edge out of buffer
        img.put_pixel(
            (image_dims.0 as f32 * particle.pos[0] - 1.0).floor() as u32,
            (image_dims.1 as f32 * particle.pos[1] - 1.0).floor() as u32,
            Luma([255u8]),
        );
    }
    let name = format!(
        "test_points_{}_{}.png",
        (layer_index as f32 / side_len as f32) as u32,
        layer_index % side_len
    );

    // save the image out
    img.save(name.clone()).unwrap();
    println!("made image {}", name);
}
