#![allow(dead_code)]
use core::panic;
use std::{io::Write, sync::mpsc::Receiver};
use pcg_with_xorshift::{PcgWithXorshift, RandomNumberGeneratorEngine};
use raw_terminal::*;

#[derive(Clone,Copy)]
enum FrontColor {
    Default,
    White,
    Black,
    Red,
    Green,
    Yellow,
    Blue,
    Purple,
    Cyan
}
impl FrontColor {
    fn encode_ascii(&self,buffer:&mut [u8])->usize{
        let color_codes = match self {
            // 31 => 0x33,0x31 is two character
            FrontColor::Default => &[0x33u8,0x39u8],
            FrontColor::Black   => &[0x33u8,0x30u8],
            FrontColor::Blue    => &[0x33u8,0x34u8],
            FrontColor::Cyan    => &[0x33u8,0x36u8],
            FrontColor::Green   => &[0x33u8,0x32u8],
            FrontColor::Purple  => &[0x33u8,0x35u8],
            FrontColor::Red     => &[0x33u8,0x31u8],
            FrontColor::White   => &[0x33u8,0x37u8],
            FrontColor::Yellow  => &[0x33u8,0x33u8],
        };
        buffer[0] = 0x1bu8;//ESC
        buffer[1] = 0x5bu8;//[
        buffer[2] = color_codes[0];
        buffer[3] = color_codes[1];
        4
    }
}
#[derive(Clone,Copy)]
enum BackColor {
    Default,
    White,
    Black,
    Red,
    Green,
    Yellow,
    Blue,
    Purple,
    Cyan
}
impl BackColor {
    fn encode_ascii(&self,buffer:&mut [u8])->usize{
        let color_codes = match self {
            BackColor::Default => &[0x34u8,0x39u8],
            BackColor::Black   => &[0x34u8,0x30u8],
            BackColor::Blue    => &[0x34u8,0x34u8],
            BackColor::Cyan    => &[0x34u8,0x36u8],
            BackColor::Green   => &[0x34u8,0x32u8],
            BackColor::Purple  => &[0x34u8,0x35u8],
            BackColor::Red     => &[0x34u8,0x31u8],
            BackColor::White   => &[0x34u8,0x37u8],
            BackColor::Yellow  => &[0x34u8,0x33u8],
        };
        buffer[0] = b';';
        buffer[1] = color_codes[0];
        buffer[2] = color_codes[1];
        buffer[3] = 0x6du8;//m
        4
    }
    fn is_default(&self)->bool{
        matches!(self, BackColor::Default)
    }
}
#[derive(Clone,Copy)]
struct Pixel{
    front_color:FrontColor,
    back_color:BackColor,
    character:char,
}
impl Default for Pixel {
    fn default() -> Self {
        Pixel { front_color: FrontColor::Default, back_color: BackColor::Default, character: ' ' }
    }
}
impl Pixel {
    fn new(c:char,fc:FrontColor,bc:BackColor)->Pixel{
        Pixel { front_color: fc, back_color: bc, character: c }
    }
    fn encode_ascii(&self,buffer:&mut [u8])->usize{
        let mut count = 0;
        count+=self.front_color.encode_ascii(&mut buffer[count..]);
        count+=self.back_color.encode_ascii(&mut buffer[count..]);
        count+=push_char_into_array(self.character, &mut buffer[count..]);
        count
    }
    fn change_character(&mut self,new_c:char){
        self.character = new_c;
    }
    fn change_front_color(&mut self,new_fc:FrontColor){
        self.front_color = new_fc;
    }
    fn change_back_color(&mut self,new_bc:BackColor){
        self.back_color = new_bc;
    }
    fn change_all(&mut self,new_c:char,new_fc:FrontColor,new_bc:BackColor){
        self.change_character(new_c);
        self.change_front_color(new_fc);
        self.change_back_color(new_bc);
    }
}
fn push_char_into_array(c:char,buffer:&mut [u8])->usize{
    c.encode_utf8(buffer).len()
}
#[derive(Clone, Copy)]
enum BlockType{
    Ttype,
    Ztype,
    Stype,
    Ltype,
    Itype,
    Otype,
}
impl BlockType {
    fn get_margin(&self)->(usize,usize){
        match self {
            BlockType::Itype => (1,2),
            BlockType::Ltype |
            BlockType::Otype => (1,1),
            BlockType::Stype => (1,0),
            BlockType::Ztype |
            BlockType::Ttype => (0,1),
        }
    }
    fn get_color(&self)->BackColor{
        match self {
            BlockType::Itype => BackColor::Blue,
            BlockType::Ltype => BackColor::Cyan,
            BlockType::Otype => BackColor::Green,
            BlockType::Stype => BackColor::Purple,
            BlockType::Ztype => BackColor::Red,
            BlockType::Ttype => BackColor::Yellow,
        }
    }
    fn random_type(rand:u32)->BlockType{
        match rand{
            0 => BlockType::Ttype,
            1 => BlockType::Ztype,
            2 => BlockType::Stype,
            3 => BlockType::Ltype,
            4 => BlockType::Itype,
            _ => BlockType::Otype
        }
    }
}
struct Blocks{
    square:Vec<Pixel>,
    t:BlockType,
    inner_left_margin:usize,
    inner_right_margin:usize,
    state:u8
}
impl Blocks {
    fn new(bt:BlockType)->Blocks{
        let margin = bt.get_margin();
        let mut temp = vec![Pixel::default();4*4];
        match bt{
            BlockType::Itype => {
                temp[1].change_back_color(bt.get_color());
                temp[5].change_back_color(bt.get_color());
                temp[9].change_back_color(bt.get_color());
                temp[13].change_back_color(bt.get_color());
            },
            BlockType::Ltype => {
                temp[14].change_back_color(bt.get_color());
                temp[5].change_back_color(bt.get_color());
                temp[9].change_back_color(bt.get_color());
                temp[13].change_back_color(bt.get_color());
            },
            BlockType::Otype =>{
                temp[10].change_back_color(bt.get_color());
                temp[14].change_back_color(bt.get_color());
                temp[9].change_back_color(bt.get_color());
                temp[13].change_back_color(bt.get_color());
            },
            BlockType::Stype => {
                temp[11].change_back_color(bt.get_color());
                temp[10].change_back_color(bt.get_color());
                temp[14].change_back_color(bt.get_color());
                temp[13].change_back_color(bt.get_color());
            },
            BlockType::Ttype => {
                temp[12].change_back_color(bt.get_color());
                temp[14].change_back_color(bt.get_color());
                temp[9].change_back_color(bt.get_color());
                temp[13].change_back_color(bt.get_color());
            },
            BlockType::Ztype => {
                temp[8].change_back_color(bt.get_color());
                temp[14].change_back_color(bt.get_color());
                temp[9].change_back_color(bt.get_color());
                temp[13].change_back_color(bt.get_color());
            }
        }
        Blocks { square: temp, t: bt,inner_left_margin:margin.0, inner_right_margin: margin.1 ,state:0}
    }
    fn trans(&mut self){
        match self.t {
            BlockType::Itype => {
                match self.state {
                    0 => {
                        for (index,pixel) in self.square.as_mut_slice().iter_mut().enumerate(){
                            if index/4 == self.inner_right_margin{
                                pixel.change_back_color(self.t.get_color());
                            }
                            else{
                                pixel.change_back_color(BackColor::Default);
                            }
                        }
                        self.inner_left_margin = 0;
                        self.inner_right_margin = 0;
                        self.state = 1;
                    }
                    _ => {
                        for (index,pixel) in self.square.as_mut_slice(). iter_mut().enumerate(){
                            if index%4 == 1{
                                pixel.change_back_color(self.t.get_color());
                            }
                            else{
                                pixel.change_back_color(BackColor::Default);
                            }
                        }
                        self.state = 0;
                        self.inner_left_margin = 1;
                        self.inner_right_margin = 2;
                    }
                }
            },
            BlockType::Otype =>{},
            BlockType::Ltype => {
                match self.state {
                    0 => {
                        let align = self.inner_left_margin/2;
                        for (index,pixel) in self.square.as_mut_slice(). iter_mut().enumerate(){
                            if index == 12+align || (index >=8+align&&index<=10+align) {
                                pixel.change_back_color(self.t.get_color());
                            }
                            else{
                                pixel.change_back_color(BackColor::Default);
                            }
                        }
                        self.inner_left_margin = align;
                        self.inner_right_margin = 1-align;
                        self.state=1;
                    },
                    1 => {
                        let align = self.inner_left_margin/2;
                        for (index,pixel) in self.square.as_mut_slice(). iter_mut().enumerate(){
                            if index == 4+align || index == 5+align || index == 9+align || index == 13+align {
                                pixel.change_back_color(self.t.get_color());
                            }
                            else{
                                pixel.change_back_color(BackColor::Default);
                            }
                        }
                        self.inner_left_margin = align;
                        self.inner_right_margin = 2-align;
                        self.state=2;
                    },
                    2 =>{
                        let align = self.inner_left_margin/2;
                        for (index,pixel) in self.square.as_mut_slice(). iter_mut().enumerate(){
                            if index == 10+align || (index >=12+align && index<=14+align) {
                                pixel.change_back_color(self.t.get_color());
                            }
                            else{
                                pixel.change_back_color(BackColor::Default);
                            }
                        }
                        self.inner_left_margin = align;
                        self.inner_right_margin = 1-align;
                        self.state=3;
                    },
                    _ => {
                        let align = self.inner_left_margin;
                        for (index,pixel) in self.square.as_mut_slice(). iter_mut().enumerate(){
                            if index == 4+align || index == 8+align || index == 12+align || index == 13+align {
                                pixel.change_back_color(self.t.get_color());
                            }
                            else{
                                pixel.change_back_color(BackColor::Default);
                            }
                        }
                        self.inner_left_margin = align;
                        self.inner_right_margin = 2-align;
                        self.state=0;
                    }
                }
            },
            BlockType::Stype => {
                match self.state {
                    0 => {
                        let align = self.inner_left_margin;
                        for (index,pixel) in self.square.as_mut_slice(). iter_mut().enumerate(){
                            if index == 4+align || index == 8+align || index == 9+align || index == 13+align {
                                pixel.change_back_color(self.t.get_color());
                            }
                            else{
                                pixel.change_back_color(BackColor::Default);
                            }
                        }
                        self.inner_left_margin = align;
                        self.inner_right_margin = 2-align;
                        self.state=1;
                    },
                    _ =>{
                        let align = self.inner_left_margin/2;
                        for (index,pixel) in self.square.as_mut_slice(). iter_mut().enumerate(){
                            if index == 12+align || index == 13+align || index == 9+align || index == 10+align {
                                pixel.change_back_color(self.t.get_color());
                            }
                            else{
                                pixel.change_back_color(BackColor::Default);
                            }
                        }
                        self.inner_left_margin = align;
                        self.inner_right_margin = 1-align;
                        self.state=0;
                    }
                }
            },
            BlockType::Ztype => {
                match self.state {
                    0 => {
                        let align = self.inner_left_margin;
                        for (index,pixel) in self.square.as_mut_slice(). iter_mut().enumerate(){
                            if index == 6+align || index == 10+align || index == 9+align || index == 13+align {
                                pixel.change_back_color(self.t.get_color());
                            }
                            else{
                                pixel.change_back_color(BackColor::Default);
                            }
                        }
                        self.inner_left_margin = align;
                        self.inner_right_margin = 2-align;
                        self.state=1;
                    },
                    _ =>{
                        let align = self.inner_left_margin/2;
                        for (index,pixel) in self.square.as_mut_slice(). iter_mut().enumerate(){
                            if index == 8+align || index == 9+align || index == 13+align || index == 14+align {
                                pixel.change_back_color(self.t.get_color());
                            }
                            else{
                                pixel.change_back_color(BackColor::Default);
                            }
                        }
                        self.inner_left_margin = align;
                        self.inner_right_margin = 1-align;
                        self.state=0;
                    }
                }
            },
            BlockType::Ttype => {
                    match self.state {
                        0 => {
                            let align = self.inner_left_margin;
                            for (index,pixel) in self.square.as_mut_slice(). iter_mut().enumerate(){
                                if index == 5+align || index == 9+align || index == 13+align || index == 10+align {
                                    pixel.change_back_color(self.t.get_color());
                                }
                                else{
                                    pixel.change_back_color(BackColor::Default);
                                }
                            }
                            self.inner_left_margin = align;
                            self.inner_right_margin = 2-align;
                            self.state=1;
                        },
                        1 =>{
                            let align = self.inner_left_margin/2;
                            for (index,pixel) in self.square.as_mut_slice(). iter_mut().enumerate(){
                                if index == 13+align || (index>=8+align&&index<=10+align) {
                                    pixel.change_back_color(self.t.get_color());
                                }
                                else{
                                    pixel.change_back_color(BackColor::Default);
                                }
                            }
                            self.inner_left_margin = align;
                            self.inner_right_margin = 1-align;
                            self.state=2;
                        },
                        2 => {
                            let align = self.inner_left_margin;
                            for (index,pixel) in self.square.as_mut_slice(). iter_mut().enumerate(){
                                if index == 8+align || index == 9+align || index == 13+align || index == 5+align {
                                    pixel.change_back_color(self.t.get_color());
                                }
                                else{
                                    pixel.change_back_color(BackColor::Default);
                                }
                            }
                            self.inner_left_margin = align;
                            self.inner_right_margin = 2-align;
                            self.state=3;
                        },
                        _ => {
                            let align = self.inner_left_margin/2;
                            for (index,pixel) in self.square.as_mut_slice(). iter_mut().enumerate(){
                                if index == 9+align || (index>=12+align&&index<=14+align) {
                                    pixel.change_back_color(self.t.get_color());
                                }
                                else{
                                    pixel.change_back_color(BackColor::Default);
                                }
                            }
                            self.inner_left_margin = align;
                            self.inner_right_margin = 1-align;
                            self.state=0;
                        }
                    }
                }
        }
    }
    fn left_and_right_move(&mut self,go_left:bool){
        if go_left{
            if self.inner_left_margin>0{
                for y in 0..4{
                    for x in 0..3{
                        self.square.as_mut_slice()[y*4+x] = self.square[y*4+x+1];
                    }
                    self.square.as_mut_slice()[y*4+3] = Pixel::default();
                }
                self.inner_left_margin-=1;
                self.inner_right_margin+=1;
            }
        }else if self.inner_right_margin>0{
            for y in 0..4{
                for x in (1..4).rev(){
                    self.square.as_mut_slice()[y*4+x] = self.square[y*4+x-1];
                }
                self.square.as_mut_slice()[y*4] = Pixel::default();
            }
            self.inner_right_margin-=1;
            self.inner_left_margin+=1;
        }
    }
}
struct Game{
    key_reader:Receiver<u8>,
    game_board:Board,
    current_block_type:BlockType,
    next_block_type:BlockType,
    pcg:PcgWithXorshift,
}
impl Game {
    fn new(reader:Receiver<u8>)->Game{
        let dimensions = get_terminal_dimensions().unwrap();
        if dimensions.0 < 17||dimensions.1<31{
            panic!("terminal dimensions too small!");
        }
        let real_dimensions = format_dimensions(dimensions);
        let mut pwxs = PcgWithXorshift::new(None);
        let ct = BlockType::random_type(pwxs.get_round(6));
        let nt = BlockType::random_type(pwxs.get_round(6));
        Game { key_reader: reader, game_board: Board::new(real_dimensions,ct), current_block_type: ct, next_block_type:nt ,pcg:pwxs}
    }
    fn run(&mut self){
        
        self.game_board.init();
        self.game_board.draw_next_block(self.next_block_type);
        self.game_board.draw_score(0);
        self.game_board.draw_speed(5);
        let mut score:u32 = 0;//max 500
        let mut speed:u32 = 5;
        let mut total_time = 100;
        let mut highest_raw = self.game_board.raws-1;
        loop {
            let mut flag = false;
            if total_time <= speed{
                self.game_board.blocks_position.1+=1;
                total_time = 100;
                flag=true;
            }
            else {
                total_time-=speed;
                let key = self.get_key_input_from_stdin();
                if key < 4{
                    self.game_board.mov(key);
                    flag = true;
                }
            }
            if self.game_board.is_bottom(){
                for y in self.game_board.blocks_position.1.saturating_sub(4)..self.game_board.blocks_position.1{
                    for x in 0..4usize{
                        if !self.game_board.blocks.square[(self.game_board.blocks_position.1-y-1)*4+x].back_color.is_default(){
                            if  y < highest_raw{
                                highest_raw = y;
                            }
                            self.game_board.matrix[(self.game_board.blocks_position.1-(y-self.game_board.blocks_position.1.saturating_sub(4)))*self.game_board.columns+(self.game_board.blocks_position.0+x)*2+2].change_back_color(self.current_block_type.get_color());
                            self.game_board.matrix[(self.game_board.blocks_position.1-(y-self.game_board.blocks_position.1.saturating_sub(4)))*self.game_board.columns+(self.game_board.blocks_position.0+x)*2+1].change_back_color(self.current_block_type.get_color());
                        }
                    }
                }
                score += self.game_board.remove_line()*10;
                if score >=500{
                    break;
                }
                if highest_raw <=1{
                    break;
                }
                speed = 5+score/100;
                self.current_block_type = self.next_block_type;
                self.game_board.blocks = Blocks::new(self.current_block_type);
                self.next_block_type = BlockType::random_type(self.pcg.get_round(6));
                self.game_board.blocks_position.0 = ((self.game_board.columns-10)/2-4)/2;
                self.game_board.blocks_position.1 = 1;

                self.game_board.draw_next_block(self.next_block_type);
                self.game_board.draw_score(score);
                self.game_board.draw_speed(speed);
                flag = true;
            }
            if flag{
                self.game_board.draw();
            }
            std::thread::sleep(std::time::Duration::from_millis(50));
        }
        reset();
        hide_cursor(false);
        set_mode(true);
        if score>=500{
            println!("You win");
        }else {
            println!("You lose");
        }
    }
    fn get_key_input_from_stdin(&self)->u8{
        if let Some(byte) = self.key_reader.try_iter().next(){
            match byte {
                b'a' => 0,
                b'd' => 1,
                b'w' => 2,
                b's' => 3,
                _ => 4
            }
        }else {
            4
        }
    }
}
struct Board{
    raws:usize,
    columns:usize,
    blocks:Blocks,
    blocks_position:(usize,usize),
    matrix:Vec<Pixel>,
    write_cache:Vec<u8>,
}
impl Board {
    fn new(dimensions:(u16,u16),current_type:BlockType)->Board{
        Board { raws: dimensions.0 as usize, columns: dimensions.1 as usize, blocks:Blocks::new(current_type),
            blocks_position:(((dimensions.1 as usize-10)/2-4)/2,1),matrix:vec![Pixel::default();(dimensions.0*dimensions.1) as usize],
            write_cache:vec![0;(dimensions.0*dimensions.1*(10)) as usize] }
    }
    fn set_pixel(&mut self,x:usize,y:usize,c:char,fc:FrontColor,bc:BackColor){
        self.matrix.as_mut_slice()[x+y*self.columns].change_all(c, fc, bc);
    }
    fn init(&mut self){
        for raw in 0..self.raws  {
            for column in 0..self.columns  {
                if raw == 0{
                    if column == 0{
                        self.set_pixel(column, raw, '┌', FrontColor::Default, BackColor::Default);
                    }
                    else if column == (self.columns-10)  {
                        self.set_pixel(column, raw, '┬', FrontColor::Default, BackColor::Default);
                    }
                    else if column == (self.columns-1)  {
                        self.set_pixel(column, raw, '┐', FrontColor::Default, BackColor::Default);
                    }else {
                        self.set_pixel(column, raw, '─', FrontColor::Default, BackColor::Default);
                    }
                }
                else if raw == (self.raws-1)  {
                    if column == 0{
                        self.set_pixel(column, raw, '└', FrontColor::Default, BackColor::Default);
                    }
                    else if column == (self.columns-10)  {
                        self.set_pixel(column, raw, '┴', FrontColor::Default, BackColor::Default);
                    }
                    else if column == (self.columns-1)  {
                        self.set_pixel(column, raw, '┘', FrontColor::Default, BackColor::Default);
                    }else {
                        self.set_pixel(column, raw, '─', FrontColor::Default, BackColor::Default);
                    }
                }
                else if column == 0 || column == (self.columns-1)   || column == (self.columns-10)  {
                    self.set_pixel(column, raw, '│', FrontColor::Default, BackColor::Default);
                }
            }
        }
    }
    fn draw_next_block(&mut self,next:BlockType){
        for (index,character) in "next:".chars().enumerate(){
            self.set_pixel(self.columns-9+index, 1, character, FrontColor::Default, BackColor::Red);
        }
        match next {
            BlockType::Itype => {
                for x in 0..6{
                    for y in 0..4{
                        if x <2{
                            self.set_pixel(self.columns-9+x, 2+y, ' ', FrontColor::Default, next.get_color());
                        }
                        else {
                            self.set_pixel(self.columns-9+x, 2+y, ' ', FrontColor::Default, BackColor::Default);
                        }
                    }
                }
            },
            BlockType::Ltype => {
                for x in 0..6{
                    for y in 0..4{
                        if (x <2 && y>=1) || (x<4 && y == 3){
                            self.set_pixel(self.columns-9+x, 2+y, ' ', FrontColor::Default, next.get_color());
                        }
                        else {
                            self.set_pixel(self.columns-9+x, 2+y, ' ', FrontColor::Default, BackColor::Default);
                        }
                    }
                }
            },
            BlockType::Otype => {
                for x in 0..6{
                    for y in 0..4{
                        if x <4 && y>1{
                            self.set_pixel(self.columns-9+x, 2+y, ' ', FrontColor::Default, next.get_color());
                        }
                        else {
                            self.set_pixel(self.columns-9+x, 2+y, ' ', FrontColor::Default, BackColor::Default);
                        }
                    }
                }
            },
            BlockType::Stype =>{
                for x in 0..6{
                    for y in 0..4{
                        if (y == 2 && x>= 2) || (x<4 && y == 3){
                            self.set_pixel(self.columns-9+x, 2+y, ' ', FrontColor::Default, next.get_color());
                        }else {
                            self.set_pixel(self.columns-9+x, 2+y, ' ', FrontColor::Default, BackColor::Default);
                        }
                    }
                }
            },
            BlockType::Ttype =>{
                for x in 0..6{
                    for y in 0..4{
                        if (x == 2||x==3) && y==2 || y==3{
                            self.set_pixel(self.columns-9+x, 2+y, ' ', FrontColor::Default, next.get_color());
                        }
                        else{
                            self.set_pixel(self.columns-9+x, 2+y, ' ', FrontColor::Default, BackColor::Default);
                        }
                    }
                }
            },
            BlockType::Ztype =>{
                for x in 0..6{
                    for y in 0..4{
                        if (x <4 && y==2) || (x>1 && y == 3){
                            self.set_pixel(self.columns-9+x, 2+y, ' ', FrontColor::Default, next.get_color());
                        }
                        else {
                            self.set_pixel(self.columns-9+x, 2+y, ' ', FrontColor::Default, BackColor::Default);
                        }
                    }
                }
            }
        }
    }
    fn draw_score(&mut self,socre:u32){
        for (index,character) in "score:".chars().enumerate(){
            self.set_pixel(self.columns-9+index, 10+(self.raws-10)/5*3, character, FrontColor::Default, BackColor::Green);
        }
        for (index,character) in socre.to_string().chars().enumerate(){
            self.set_pixel(self.columns-9+index, 10+(self.raws-10)/5*3+1, character, FrontColor::Yellow, BackColor::Default);
        }
    }
    fn draw_speed(&mut self,speed:u32){
        for (index,character) in "speed:".chars().enumerate(){
            self.set_pixel(self.columns-9+index, 10+(self.raws-10)/5*4, character, FrontColor::Default, BackColor::Blue);
        }
        for (index,character) in speed.to_string().chars().enumerate(){
            self.set_pixel(self.columns-9+index, 10+(self.raws-10)/5*4+1, character, FrontColor::Yellow, BackColor::Default);
        }
    }
    fn draw(&mut self){
        let mut position = 0;
        for (index,pixel) in self.matrix.iter().enumerate(){
            let (x,y) = (index%self.columns,index/self.columns);
            if x>0 && x<self.columns-10 && y >0 && y<self.raws-1{
                let real_x = (x-1)/2;
                if real_x >= self.blocks_position.0 && real_x<= self.blocks_position.0+3 && y <= self.blocks_position.1 && y>= self.blocks_position.1.checked_sub(3).unwrap_or(1){
                    if self.blocks.square[real_x-self.blocks_position.0+(3+y-self.blocks_position.1)*4].back_color.is_default(){
                        position+=pixel.encode_ascii(&mut self.write_cache[position..]);
                    }else {
                        position+=self.blocks.square[real_x-self.blocks_position.0+(3+y-self.blocks_position.1)*4].encode_ascii(&mut self.write_cache[position..]);
                    }
                }else {
                    position+=pixel.encode_ascii(&mut self.write_cache[position..]);
                }
            }
            else {
                position+=pixel.encode_ascii(&mut self.write_cache[position..]);
            }
            if index%self.columns == self.columns-1 && index < self.columns*self.raws-1{
                position+=push_char_into_array('\n', &mut self.write_cache[position..]);
            }
        }
        reset();
        std::io::stdout().write_all(&self.write_cache).unwrap();
        std::io::stdout().flush().unwrap();
    }
    fn is_fill_line(&self,line_num:usize)->bool{
        for x in 1..self.columns-10{
            if self.matrix[line_num*self.columns+x].back_color.is_default(){
                return false;
            }
        }
        true
    }
    fn remove_line(&mut self)->u32{
        let mut jump_num:usize = 0;
        let mut y = self.raws-1;
        while y>0{
            if y>jump_num{
                if self.is_fill_line(y-jump_num){
                    jump_num+=1;
                    continue;
                }else if jump_num>0{
                    for x in 1..(self.columns-10){
                        self.matrix[y*self.columns+x] = self.matrix[(y-jump_num)*self.columns+x];
                    }
                }
            }
            else {
                for x in 1..(self.columns-10){
                    self.matrix[y*self.columns+x].change_back_color(BackColor::Default);
                }
            }
            y-=1;
        }
        jump_num as u32
    }
    fn is_bottom(&mut self)->bool{
        for x in 0..4{
            if (!self.blocks.square[12+x].back_color.is_default())&&
            (self.matrix[(self.blocks_position.1+1)*self.columns+(self.blocks_position.0+x)*2+1].character !=' ' ||
            !self.matrix[(self.blocks_position.1+1)*self.columns+(self.blocks_position.0+x)*2+1].back_color.is_default()||
            self.matrix[(self.blocks_position.1+1)*self.columns+(self.blocks_position.0+x)*2+2].character !=' ' ||
            !self.matrix[(self.blocks_position.1+1)*self.columns+(self.blocks_position.0+x)*2+2].back_color.is_default()
            ){
                return true;
            }
            if self.blocks_position.1>1 && (!self.blocks.square[8+x].back_color.is_default())&&
            (self.matrix[(self.blocks_position.1)*self.columns+(self.blocks_position.0+x)*2+1].character !=' ' ||
            !self.matrix[(self.blocks_position.1)*self.columns+(self.blocks_position.0+x)*2+1].back_color.is_default()||
            self.matrix[(self.blocks_position.1)*self.columns+(self.blocks_position.0+x)*2+2].character !=' ' ||
            !self.matrix[(self.blocks_position.1)*self.columns+(self.blocks_position.0+x)*2+2].back_color.is_default()
            ){
                return true;
            }
            if self.blocks_position.1>2 && (!self.blocks.square[4+x].back_color.is_default())&&
            (self.matrix[(self.blocks_position.1-1)*self.columns+(self.blocks_position.0+x)*2+1].character !=' ' ||
            !self.matrix[(self.blocks_position.1-1)*self.columns+(self.blocks_position.0+x)*2+1].back_color.is_default()||
            self.matrix[(self.blocks_position.1-1)*self.columns+(self.blocks_position.0+x)*2+2].character !=' ' ||
            !self.matrix[(self.blocks_position.1-1)*self.columns+(self.blocks_position.0+x)*2+2].back_color.is_default()
            ){
                return true;
            }
        }
        false
    }
    /// direction
    /// 0  left
    /// 1  right
    /// 2  up
    /// 3  down
    fn mov(&mut self,direction:u8){
        
        match direction {
           0 => {
             if self.blocks_position.0>0{
                let mut flag = true;
                for y in self.blocks_position.1.saturating_sub(4)..self.blocks_position.1{
                    if !self.matrix[(1+y)*self.columns+(self.blocks_position.0+self.blocks.inner_left_margin)*2].back_color.is_default() && !self.blocks.square[16-(self.blocks_position.1-y)*4+self.blocks.inner_left_margin].back_color.is_default(){
                        flag=false;
                        break;
                    }
                }
                if flag{
                    self.blocks_position.0-=1;
                }
             }
             else {
                 self.blocks.left_and_right_move(true);
             }
           },
           1 => {
            if self.blocks_position.0<(self.columns-10)/2-4{
                let mut flag = true;
                for y in self.blocks_position.1.saturating_sub(4)..self.blocks_position.1{
                    if !self.matrix[(1+y)*self.columns+(self.blocks_position.0-self.blocks.inner_right_margin)*2+9].back_color.is_default() && !self.blocks.square[16-(self.blocks_position.1-y)*4+3-self.blocks.inner_right_margin].back_color.is_default(){
                        flag=false;
                        break;
                    }
                }
                if flag{
                    self.blocks_position.0+=1;
                }
            }else {
                self.blocks.left_and_right_move(false);
            }
           },
           2 => {
            self.blocks.trans();
           },
           _ =>{
            self.blocks_position.1+=1;
           }
        }
    }
}
/// user should confirm the input dimensions bigger than 31x17
fn format_dimensions(dim:(u16,u16))->(u16,u16){
    if (dim.0 - 2)*2 <= dim.1-11{
        (dim.0,(dim.0-2)*2+3+8)
    }else if dim.1%2==1{
        ((dim.1-11)/2+2,dim.1)
    }else {
        ((dim.1-12)/2+2,dim.1-1)
    }
}
fn main() {
    set_mode(false);
    reset();
    hide_cursor(true);
    let (s,recv) = std::sync::mpsc::channel::<u8>();
    std::thread::spawn(move || {
        use std::io::Read;
        let mut stdin = std::io::stdin();
        let mut buffer = [0u8; 1];
        loop {
            stdin.read_exact(&mut buffer).unwrap();
            s.send(*buffer.first().unwrap()).unwrap();
        }
    });
    let mut game = Game::new(recv);
    game.run();
}