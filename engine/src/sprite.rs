use canvas::Canvas;
//精灵代码
pub type SPRITEACTION = u32;
pub const SA_NONE: SPRITEACTION = 0;
pub const SA_KILL: SPRITEACTION = 1;
pub const SA_ADDSPRITE: SPRITEACTION = 2;

pub type BOUNDSACTION = u32;
pub const BA_STOP: BOUNDSACTION = 0;
pub const BA_WRAP: BOUNDSACTION = 1;
pub const BA_BOUNCE: BOUNDSACTION = 2;
pub const BA_DIE: BOUNDSACTION = 3;
use Bitmap;

#[derive(Clone, Debug, Copy)]
pub struct Rect {
    pub left: f64,
    pub top: f64,
    pub right: f64,
    pub bottom: f64,
}

impl Rect {
    pub fn new(left: f64, top: f64, right: f64, bottom: f64) -> Rect {
        Rect {
            left,
            top,
            right,
            bottom,
        }
    }

    pub fn zero() -> Rect {
        Rect {
            left: 0.0,
            top: 0.0,
            right: 0.0,
            bottom: 0.0,
        }
    }

    /** 修改rect大小 */
    pub fn inflate(&mut self, dx: f64, dy: f64) {
        self.left -= dx;
        self.right += dx;
        self.top -= dy;
        self.bottom += dy;
    }

    pub fn offset(&mut self, dx: f64, dy: f64) {
        self.left += dx;
        self.right += dx;
        self.top += dy;
        self.bottom += dy;
    }

    pub fn contain(&self, x: f64, y: f64) -> bool {
        x >= self.left && x <= self.right && y >= self.top && y <= self.bottom
    }
}

#[derive(Clone, Debug, Copy)]
pub struct PointF {
    pub x: f64,
    pub y: f64,
}

impl PointF {
    pub fn new(x: f64, y: f64) -> PointF {
        PointF { x: x, y: y }
    }

    pub fn zero() -> PointF {
        PointF { x: 0.0, y: 0.0 }
    }
}

#[derive(Clone, Debug, Copy)]
pub struct Point {
    pub x: i32,
    pub y: i32,
}

impl Point {
    pub fn new() -> Point {
        Point { x: 0, y: 0 }
    }
}

pub trait Sprite{
    fn draw(&self, context: &Canvas);
    fn z_order(&self) -> i32;
    fn position(&self) -> &Rect;
    fn update(&mut self, elapsed_milis: f64) -> SPRITEACTION;
    fn id(&self) -> u32;
    fn set_position_rect(&mut self, position: Rect);
    fn test_collison(&self, test: &Rect) -> bool;
    fn kill(&mut self);
}

pub struct Entity {
    pub id: u32,
    pub parent_id: u32,
    bitmap: Box<Bitmap>,
    num_frames: i32,
    pub cur_frame: i32,
    frame_delay: i32,
    frame_trigger: i32,
    pub position: Rect,
    target_position: Option<PointF>,
    bounds: Rect,
    velocity: PointF,
    pub z_order: i32,
    collision: Rect,
    bounds_action: BOUNDSACTION,
    hidden: bool,
    pub dying: bool,
    one_cycle: bool,
    name: String,
    score: i32,
    killer_id: u32,
    killer_name: String,
    lives: u32,
    rotation: f64,
}

impl Entity {
    pub fn new(
        id: u32,
        bitmap: Box<Bitmap>,
        position: PointF,
        velocity: PointF,
        z_order: i32,
        bounds: Rect,
        bounds_action: BOUNDSACTION,
    ) -> Entity {
        let mut sprite = Entity {
            id: id,
            parent_id: 0,
            position: Rect::new(
                position.x,
                position.y,
                position.x + bitmap.width() as f64,
                position.y + bitmap.height() as f64,
            ),
            target_position: None,
            bitmap: bitmap,
            num_frames: 1,
            cur_frame: 0,
            frame_delay: 0,
            frame_trigger: 0,
            velocity: velocity,
            z_order: z_order,
            bounds: bounds,
            bounds_action: bounds_action,
            hidden: false,
            dying: false,
            one_cycle: false,
            name: "".to_string(),
            collision: Rect::zero(),
            score: 0,
            killer_id: 0,
            killer_name: String::new(),
            lives: 0,
            rotation: 0.0,
        };
        sprite.calc_collision_rect();
        sprite
    }

    pub fn from_bitmap(id: u32, bitmap: Box<Bitmap>, bounds: Rect) -> Entity {
        Entity::new(
            id,
            bitmap,
            PointF::zero(),
            PointF::zero(),
            0,
            bounds,
            BA_STOP,
        )
    }

    pub fn with_bounds_action(
        id: u32,
        bitmap: Box<Bitmap>,
        position: PointF,
        bounds: Rect,
        bounds_action: BOUNDSACTION,
    ) -> Entity {
        Entity::new(
            id,
            bitmap,
            position,
            PointF::zero(),
            0,
            bounds,
            bounds_action,
        )
    }

    // pub fn with_bounds_action_norand(
    //     id: u32,
    //     bitmap: BitmapRes,
    //     bounds: Rect,
    //     bounds_action: BOUNDSACTION,
    // ) -> Sprite {
    //     Sprite::new(
    //         id,
    //         bitmap,
    //         PointF::new(),
    //         PointF::new(),
    //         0,
    //         bounds,
    //         bounds_action,
    //     )
    // }

    fn calc_collision_rect(&mut self) {
        let x_shrink = (self.position.left - self.position.right) / 12.0;
        let y_shrink = (self.position.top - self.position.bottom) / 12.0;
        self.collision = self.position;
        self.collision.inflate(x_shrink, y_shrink);
    }

    //-----------------------------------------------------------------
    // Sprite General Methods
    //-----------------------------------------------------------------
    pub fn update(&mut self, elapsed_milis: f64) -> SPRITEACTION {
        // See if the sprite needs to be killed
        if self.dying {
            return SA_KILL;
        }

        // Update the frame
        self.update_frame();

        //检查是否到达目标位置
        if let Some(target) = self.target_position {
            if self.velocity.x == 0.0 && self.velocity.y == 0.0 {
                if target.x != self.position.left || target.y != self.position.top {
                    self.set_position_point(&PointF {
                        x: target.x,
                        y: target.y,
                    });
                }
            }
        }

        // if let Some((target, velocity)) = self.target{
        //     let mut tmp_position = PointF{
        //         x: self.position.left,
        //         y: self.position.top,
        //     };
        //     self.velocity.x = velocity.x;
        //     self.velocity.y = velocity.y;
        //     if self.velocity.x != 0.0 && self.velocity.y != 0.0{
        //         self.last_velocity  = Some(velocity);
        //     }
        //     //由于每次绘制已经过去几十ms, 精灵有可能越过目标点, 所以这里进一步计算
        //     let mut distance = 0.0;
        //     for _ in 0..elapsed_milis as u32{
        //         tmp_position.x += self.velocity.x;
        //         tmp_position.y += self.velocity.y;
        //         let (dx, dy) = (target.x - tmp_position.x, target.y - tmp_position.y);
        //         distance =  (dx * dx + dy * dy).sqrt();
        //         //达到目标点(这里的1.0是假设游戏中最快的精灵速度不超过1.0)
        //         if distance.abs()<1.0{
        //             self.velocity.x = 0.0;
        //             self.velocity.y = 0.0;
        //             break;
        //         }else if distance.abs()>100.0{
        //             //正常情况下延迟不会导致距离差距到100
        //             //精灵穿越墙的时候，会导致服务器和客户端距离为整个屏幕的宽度或者高度，这时候不进行移动，直接跳过去
        //             self.velocity.x = 0.0;
        //             self.velocity.y = 0.0;
        //             self.set_position_point(&PointF{
        //                 x: target.x,
        //                 y: target.y,
        //             });
        //             break;
        //         }
        //     }
        //     //如果距离仍然很大，但是速度为零，这时候也直接将精灵移动过去
        //     if velocity.x == 0.0 && velocity.y == 0.0 && distance>1.0{
        //         if self.last_velocity.is_none(){
        //             self.set_position_point(&PointF{
        //                 x: target.x,
        //                 y: target.y,
        //             });
        //         }else{
        //             //如果存在上次移动的速度，按照最后一次速度移动
        //             self.velocity = self.last_velocity.unwrap();
        //         }
        //     }
        // }

        //Update the position
        let mut new_position = PointF::zero();
        let mut sprite_size = PointF::zero();
        let mut bounds_size = PointF::zero();

        new_position.x = self.position.left + self.velocity.x * elapsed_milis;
        new_position.y = self.position.top + self.velocity.y * elapsed_milis;
        sprite_size.x = self.position.right - self.position.left;
        sprite_size.y = self.position.bottom - self.position.top;
        bounds_size.x = self.bounds.right - self.bounds.left;
        bounds_size.y = self.bounds.bottom - self.bounds.top;

        // Check the bounds
        // Wrap?
        if self.bounds_action == BA_WRAP {
            if (new_position.x + sprite_size.x) < self.bounds.left {
                new_position.x = self.bounds.right;
            } else if new_position.x > self.bounds.right {
                new_position.x = self.bounds.left - sprite_size.x;
            }
            if (new_position.y + sprite_size.y) < self.bounds.top {
                new_position.y = self.bounds.bottom;
            } else if new_position.y > self.bounds.bottom {
                new_position.y = self.bounds.top - sprite_size.y;
            }
        }
        // Bounce?
        else if self.bounds_action == BA_BOUNCE {
            let mut bounce = false;
            let mut new_velocity = self.velocity;
            if new_position.x < self.bounds.left {
                bounce = true;
                new_position.x = self.bounds.left;
                new_velocity.x = -new_velocity.x;
            } else if (new_position.x + sprite_size.x) > self.bounds.right {
                bounce = true;
                new_position.x = self.bounds.right - sprite_size.x;
                new_velocity.x = -new_velocity.x;
            }
            if new_position.y < self.bounds.top {
                bounce = true;
                new_position.y = self.bounds.top;
                new_velocity.y = -new_velocity.y;
            } else if (new_position.y + sprite_size.y) > self.bounds.bottom {
                bounce = true;
                new_position.y = self.bounds.bottom - sprite_size.y;
                new_velocity.y = -new_velocity.y;
            }
            if bounce {
                self.velocity = new_velocity;
            }
        }
        // Die?
        else if self.bounds_action == BA_DIE {
            if (new_position.x + sprite_size.x) < self.bounds.left
                || new_position.x > self.bounds.right
                || (new_position.y + sprite_size.y) < self.bounds.top
                || new_position.y > self.bounds.bottom
            {
                return SA_KILL;
            }
        }
        // Stop (default)
        else {
            if new_position.x < self.bounds.left
                || new_position.x > (self.bounds.right - sprite_size.x)
            {
                new_position.x = f64::max(
                    self.bounds.left,
                    f64::min(new_position.x, self.bounds.right - sprite_size.x),
                );
                self.set_velocity(0.0, 0.0);
            }
            if new_position.y < self.bounds.top
                || new_position.y > (self.bounds.bottom - sprite_size.y)
            {
                new_position.y = f64::max(
                    self.bounds.top,
                    f64::min(new_position.y, self.bounds.bottom - sprite_size.y),
                );
                self.set_velocity(0.0, 0.0);
            }
        }
        self.set_position_point(&new_position);

        //let msg = format!("after update>position={:?}", self.position());
        //unsafe { log(msg.as_ptr(), msg.len()); }
        SA_NONE
    }

    pub fn draw(&self, context: &Canvas) {
        // Draw the sprite if it isn't hidden
        if !self.hidden {
            // Draw the appropriate frame, if necessary
            match self.num_frames {
                1 => context.draw_image_at(
                    self.bitmap.as_ref(),
                    self.position.left as i32,
                    self.position.top as i32,
                ),
                _ => context.draw_image(
                    self.bitmap.as_ref(),
                    0,
                    self.cur_frame * self.height(),
                    self.width(),
                    self.height(),
                    self.position.left as i32,
                    self.position.top as i32,
                    self.width(),
                    self.height(),
                ),
            }
            context.fill_style("#ccccff");
            context.set_font("16px 微软雅黑");
            if self.name.len() > 0 && self.score >= 0 {
                let score = &format!("({}分)", self.score);
                let w = self.name.len() * 5 + score.len() * 5;
                let x = self.position.left as i32
                    + ((self.position.right - self.position.left) as i32 / 2 - (w as i32 / 2));
                let y = self.position.bottom as i32 + 20;
                context.fill_text(&format!("{}{}", self.name, score), x, y);
            }
            //绘制坦克生命值
            let mut lives = String::new();
            for _ in 0..self.lives {
                //lives.push_str("❤️");
                lives.push_str("♡");
            }
            context.fill_style(if self.lives > 3 { "#ffff00" } else { "#ff0000" });
            context.fill_text(
                &lives,
                self.position.left as i32,
                self.position.bottom as i32 + 40,
            );
        }
    }

    pub fn update_frame(&mut self) {
        self.frame_trigger -= 1;
        if (self.frame_delay >= 0) && (self.frame_trigger <= 0) {
            // Reset the frame trigger;
            self.frame_trigger = self.frame_delay;

            // Increment the frame
            self.cur_frame += 1;
            if self.cur_frame >= self.num_frames {
                // If it's a one-cycle frame animation, kill the sprite
                match self.one_cycle {
                    true => self.dying = true,
                    _ => self.cur_frame = 0,
                }
            }
        }
    }

    pub fn set_frame_delay(&mut self, frame_delay: i32) {
        self.frame_delay = frame_delay;
    }

    pub fn set_velocity(&mut self, x: f64, y: f64) {
        self.velocity.x = x;
        self.velocity.y = y;
    }

    pub fn set_velocity_point(&mut self, velocity: &PointF) {
        self.velocity.x = velocity.x;
        self.velocity.y = velocity.y;
    }

    pub fn velocity(&self) -> &PointF {
        &self.velocity
    }

    pub fn set_position_point(&mut self, position: &PointF) {
        let dx = position.x - self.position.left;
        let dy = position.y - self.position.top;
        self.position.offset(dx, dy);
        self.calc_collision_rect();
    }

    pub fn set_position(&mut self, x: f64, y: f64) {
        let x = x - self.position.left;
        let y = y - self.position.top;
        self.position.offset(x, y);
        self.calc_collision_rect();
    }

    pub fn test_collison(&self, test: &Rect) -> bool {
        self.collision.left <= test.right && test.left <= self.collision.right
            && self.collision.top <= test.bottom && test.top <= self.collision.bottom
    }

    pub fn is_point_inside(&self, x: f64, y: f64) -> bool {
        self.position.contain(x, y)
    }

    pub fn height(&self) -> i32 {
        if self.num_frames > 0 {
            self.bitmap.height() / self.num_frames
        } else {
            self.bitmap.height()
        }
    }

    pub fn width(&self) -> i32 {
        self.bitmap.width()
    }

    pub fn bitmap(&self) -> &Box<Bitmap> {
        &self.bitmap
    }

    pub fn hidden(&self) -> bool {
        self.hidden
    }

    pub fn name(&self) -> &String {
        &self.name
    }

    pub fn set_name(&mut self, name: String) {
        self.name = name;
    }

    pub fn set_killer(&mut self, killer_id: u32, killer_name: String) {
        self.killer_id = killer_id;
        self.killer_name = killer_name;
    }

    pub fn killer_id(&self) -> u32 {
        self.killer_id
    }

    pub fn killer_name(&self) -> &String {
        &self.killer_name
    }

    pub fn set_num_frames(&mut self, num_frames: i32, one_cycle: bool) {
        self.num_frames = num_frames;
        self.one_cycle = one_cycle;

        //重新计算位置
        self.position.bottom =
            self.position.top + (self.position.bottom - self.position.top) / self.num_frames as f64;
    }

    pub fn dying(&self) -> bool {
        self.dying
    }

    pub fn add_score(&mut self) {
        self.score += 1;
    }

    pub fn set_score(&mut self, score: i32) {
        self.score = score;
    }

    pub fn score(&self) -> i32 {
        self.score
    }

    pub fn set_lives(&mut self, lives: u32) {
        self.lives = lives;
    }

    pub fn lives(&self) -> u32 {
        self.lives
    }

    pub fn set_rotation(&mut self, rotation: f64) {
        self.rotation = rotation;
    }

    pub fn set_target_position(&mut self, target: PointF) {
        self.target_position = Some(target);
    }
}
