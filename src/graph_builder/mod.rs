use image::{self, RgbImage};

pub fn basic_graph_builder(data: Vec<f64>) -> RgbImage{
    return basic_graph_canvas(1080, 1080);
}

fn basic_graph_canvas(width: u32, height: u32) -> RgbImage{
    let mut image_vec: Vec<u8> = vec![255; (width*height*3).try_into().unwrap()];
    let segment_separation = width/7;

    for y in 0..height{
        for x in 0..width{
            let primary_line_center = x%segment_separation;

            if primary_line_center < 3
            {   
                image_vec[(x*3 + 0 + width*3*y) as usize] = 190; 
                image_vec[(x*3 + 1 + width*3*y) as usize] = 190; 
                image_vec[(x*3 + 2 + width*3*y) as usize] = 190;
            }

            if primary_line_center == 10*(segment_separation/24)
            {
                image_vec[(x*3 + 0 + width*3*y) as usize] = 190; 
                image_vec[(x*3 + 1 + width*3*y) as usize] = 190; 
                image_vec[(x*3 + 2 + width*3*y) as usize] = 190;
            }

            if primary_line_center == (33*segment_separation)/48{

                image_vec[(x*3 + 0 + width*3*y) as usize] = 190; 
                image_vec[(x*3 + 1 + width*3*y) as usize] = 190; 
                image_vec[(x*3 + 2 + width*3*y) as usize] = 190;
            }

            if primary_line_center == 18*(segment_separation/24){
                image_vec[(x*3 + 0 + width*3*y) as usize] = 190; 
                image_vec[(x*3 + 1 + width*3*y) as usize] = 190; 
                image_vec[(x*3 + 2 + width*3*y) as usize] = 190; 
            }

            if primary_line_center == 20*(segment_separation/24)
            {
                image_vec[(x*3 + 0 + width*3*y) as usize] = 190; 
                image_vec[(x*3 + 1 + width*3*y) as usize] = 190; 
                image_vec[(x*3 + 2 + width*3*y) as usize] = 190;
            }
        }
    }
    return RgbImage::from_raw(width, height, image_vec).unwrap();
}
