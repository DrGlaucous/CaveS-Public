use crate::common::Rect;
use crate::framework::context::Context;
use crate::framework::error::GameResult;
use crate::game::frame::Frame;
use crate::game::shared_game_state::{SharedGameState, TileSize};
use crate::game::stage::{BackgroundType, Stage, StageTexturePaths};

pub struct Tilemap {
    tick: u32,
    prev_tick: u32,
    pub no_water: bool,
}

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum TileLayer {
    Background,
    Middleground,
    Foreground,
    FarForeground,
    Snack,
}

impl Tilemap {
    pub fn new() -> Self {
        Tilemap { tick: 0, prev_tick: 0, no_water: false }
    }

    pub fn tick(&mut self) -> GameResult {
        self.tick = self.tick.wrapping_add(1);
        Ok(())
    }

    pub fn set_prev(&mut self) -> GameResult {
        self.prev_tick = self.tick;
        Ok(())
    }

    pub fn draw(
        &self,
        state: &mut SharedGameState,
        ctx: &mut Context,
        frame: &Frame,
        layer: TileLayer,
        textures: &StageTexturePaths,
        stage: &Stage,
    ) -> GameResult {
        if stage.map.tile_size == TileSize::Tile8x8 && layer == TileLayer::Snack {
            return Ok(());
        }

        //tileset texture to use when drawing
        let tex = match layer {
            TileLayer::Snack => "Npc/NpcSym",
            TileLayer::Background => &textures.tileset_bg,
            TileLayer::Middleground => &textures.tileset_mg,
            TileLayer::Foreground => &textures.tileset_fg,
            _ => &textures.tileset_fg, //this will only be used with the PXM type, so this string won't matter anyway
        };

        //if pxpack data exists, load in width and height from corresponding layer
        //if not, load it from default map size
        let (layer_offset, layer_width, layer_height, uses_layers) = if let Some(pxpack_data) = &stage.data.pxpack_data
        {
            match layer {
                TileLayer::Background => {
                    (pxpack_data.offset_bg as usize, pxpack_data.size_bg.0, pxpack_data.size_bg.1, true)
                }
                TileLayer::Middleground => {
                    (pxpack_data.offset_mg as usize, pxpack_data.size_mg.0, pxpack_data.size_mg.1, true)
                }
                TileLayer::FarForeground =>{
                    return Ok(()); //do not attempt to draw the far foreground if our layers are from a pxpack
                }
                _ => (0, pxpack_data.size_fg.0, pxpack_data.size_fg.1, true), //foreground or snack
            }
        } else if stage.map.tiles.len() > (stage.map.width * stage.map.height) as usize //PXM layer mode detection
            {
                //layer order:
                //0 foreground
                //1 far back
                //2 back
                //3 far front
                match layer {
                    TileLayer::Background =>(1 * (stage.map.width * stage.map.height) as usize, stage.map.width, stage.map.height, true),
                    TileLayer::Middleground =>(2 * (stage.map.width * stage.map.height) as usize, stage.map.width, stage.map.height, true),
                    TileLayer::Foreground =>(0 * (stage.map.width * stage.map.height) as usize, stage.map.width, stage.map.height, true),
                    TileLayer::FarForeground =>(3 * (stage.map.width * stage.map.height) as usize, stage.map.width, stage.map.height, true),
                    TileLayer::Snack =>(0 * (stage.map.width * stage.map.height) as usize, stage.map.width, stage.map.height, false), //do this so we bypass the first draw-all section of the match statement below
                }

        } else {
            (0, stage.map.width, stage.map.height, false)
        };

        //do not draw mid-ground tiles if layers are not turned on
        if !uses_layers && (layer == TileLayer::Middleground || layer == TileLayer::FarForeground) {
            return Ok(());
        }

        //for integer and floating point camera positioning, respectively
        let tile_size = state.tile_size.as_int();
        let tile_sizef = state.tile_size.as_float();
        let halft = tile_size / 2;
        let halftf = tile_sizef / 2.0;

        let batch = state.texture_set.get_or_load_batch(ctx, &state.constants, tex)?;
        let mut rect = Rect::new(0, 0, tile_size as u16, tile_size as u16);
        let (mut frame_x, mut frame_y) = frame.xy_interpolated(state.frame_time);

        //what is PxPackScroll?
        if let Some(pxpack_data) = &stage.data.pxpack_data {
            let (fx, fy) = match layer {
                TileLayer::Background => pxpack_data.scroll_bg.transform_camera_pos(frame_x, frame_y),
                TileLayer::Middleground => pxpack_data.scroll_mg.transform_camera_pos(frame_x, frame_y),
                _ => pxpack_data.scroll_fg.transform_camera_pos(frame_x, frame_y),
            };

            frame_x = fx;
            frame_y = fy;
        }

        //set up picker bounds for just the tiles that are within the seen frame
        let tile_start_x = (frame_x as i32 / tile_size).clamp(0, layer_width as i32) as usize;
        let tile_start_y = (frame_y as i32 / tile_size).clamp(0, layer_height as i32) as usize;
        let tile_end_x =
            ((frame_x as i32 + 8 + state.canvas_size.0 as i32) / tile_size + 1).clamp(0, layer_width as i32) as usize;
        let tile_end_y = ((frame_y as i32 + halft + state.canvas_size.1 as i32) / tile_size + 1)
            .clamp(0, layer_height as i32) as usize;

        //set rect to snack tile
        if layer == TileLayer::Snack {
            rect = state.constants.world.snack_rect;
        }

        //choose what to draw where (for each column in each row in the viewed screen)
        for y in tile_start_y..tile_end_y {
            for x in tile_start_x..tile_end_x {
                let tile = *stage.map.tiles.get((y * layer_width as usize) + x + layer_offset).unwrap();
                match layer {
                    //all cases first check if we are using layers. If so, we ignore the attribute hardcode and draw everything except NULL tiles
                    _ if uses_layers => {
                        if tile == 0 {
                            continue;
                        }

                        let tile_size = tile_size as u16;
                        rect.left = (tile as u16 % 16) * tile_size;
                        rect.top = (tile as u16 / 16) * tile_size;
                        rect.right = rect.left + tile_size;
                        rect.bottom = rect.top + tile_size;
                    }
                    TileLayer::Background => {
                        if stage.map.attrib[tile as usize] >= 0x20 {
                            continue;
                        }

                        let tile_size = tile_size as u16;
                        rect.left = (tile as u16 % 16) * tile_size;
                        rect.top = (tile as u16 / 16) * tile_size;
                        rect.right = rect.left + tile_size;
                        rect.bottom = rect.top + tile_size;
                    }
                    TileLayer::Foreground => {
                        let attr = stage.map.attrib[tile as usize];

                        if attr < 0x40 || attr >= 0x80 {
                            continue;
                        }

                        let tile_size = tile_size as u16;
                        rect.left = (tile as u16 % 16) * tile_size;
                        rect.top = (tile as u16 / 16) * tile_size;
                        rect.right = rect.left + tile_size;
                        rect.bottom = rect.top + tile_size;
                    }
                    TileLayer::Snack => {
                        if stage.map.attrib[tile as usize] != 0x43 {
                            continue;
                        }
                    }
                    _ => {}
                }

                batch.add_rect(
                    (x as f32 * tile_sizef - halftf) - frame_x,
                    (y as f32 * tile_sizef - halftf) - frame_y,
                    &rect,
                );
            }
        }

        batch.draw(ctx)?;

        if !self.no_water && layer == TileLayer::Foreground && stage.data.background_type == BackgroundType::Water {
            let batch = state.texture_set.get_or_load_batch(ctx, &state.constants, &textures.background)?;
            let rect_top = Rect { left: 0, top: 0, right: 32, bottom: 16 };
            let rect_middle = Rect { left: 0, top: 16, right: 32, bottom: 48 };

            let tile_start_x = frame_x as i32 / 32;
            let tile_end_x = (frame_x + 16.0 + state.canvas_size.0) as i32 / 32 + 1;
            let water_y = state.water_level as f32 / 512.0;
            let tile_count_y = (frame_y + 16.0 + state.canvas_size.1 - water_y) as i32 / 32 + 1;

            for x in tile_start_x..tile_end_x {
                batch.add_rect((x as f32 * 32.0) - frame_x, water_y - frame_y, &rect_top);

                for y in 0..tile_count_y {
                    batch.add_rect((x as f32 * 32.0) - frame_x, (y as f32 * 32.0) + water_y - frame_y, &rect_middle);
                }
            }

            batch.draw(ctx)?;
        }

        if layer == TileLayer::Foreground {
            let batch = state.texture_set.get_or_load_batch(ctx, &state.constants, "Caret")?;

            for y in tile_start_y..tile_end_y {
                for x in tile_start_x..tile_end_x {
                    let tile = *stage.map.tiles.get((y * layer_width as usize) + x + layer_offset).unwrap();
                    let attr = stage.map.attrib[tile as usize];

                    if ![0x80, 0x81, 0x82, 0x83, 0xA0, 0xA1, 0xA2, 0xA3].contains(&attr) {
                        continue;
                    }

                    let shift =
                        ((self.tick as f64 + (self.tick - self.prev_tick) as f64 * state.frame_time) * 2.0) as u16 % 16;
                    let mut push_rect = state.constants.world.water_push_rect;

                    match attr {
                        0x80 | 0xA0 => {
                            push_rect.left = push_rect.left + shift;
                            push_rect.right = push_rect.right + shift;
                        }
                        0x81 | 0xA1 => {
                            push_rect.top = push_rect.top + shift;
                            push_rect.bottom = push_rect.bottom + shift;
                        }
                        0x82 | 0xA2 => {
                            push_rect.left = push_rect.left - shift + state.tile_size.as_int() as u16;
                            push_rect.right = push_rect.right - shift + state.tile_size.as_int() as u16;
                        }
                        0x83 | 0xA3 => {
                            push_rect.top = push_rect.top - shift + state.tile_size.as_int() as u16;
                            push_rect.bottom = push_rect.bottom - shift + state.tile_size.as_int() as u16;
                        }
                        _ => (),
                    }

                    batch.add_rect(
                        (x as f32 * tile_sizef - halftf) - frame_x,
                        (y as f32 * tile_sizef - halftf) - frame_y,
                        &push_rect,
                    );
                }
            }

            batch.draw(ctx)?;
        }

        Ok(())
    }
}
