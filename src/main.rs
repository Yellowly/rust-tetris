use std::{collections::VecDeque, io::BufRead};

use yew::prelude::*;
use web_sys::{HtmlInputElement, HtmlCanvasElement, TouchEvent, Element};
use wasm_bindgen::{JsCast, JsValue};
use gloo::{console::{self, Timer, dirxml}, timers::callback};
use gloo::timers::callback::{Interval, Timeout};
use rand::Rng;

fn main() {
    println!("Hello, world!");
    yew::start_app::<RootComponent>();
}

struct RootComponent{

}
impl Component for RootComponent{
    type Message = ();

    type Properties = ();

    fn create(ctx: &Context<Self>) -> Self {
        Self{}
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        html!{
            <div class="root">
                <h1>{"test"}</h1>
                <hr/>
                <GameDisplay/>
            </div>
        }
    }
}

enum GameMsg {
    Left(bool),
    Right(bool),
    Down,
    Drop,
    Tick,
    Hold,
    Rotate,
    CancelDown,
    CancelRight,
    CancelLeft,
    None
}

struct GameDisplay{
    game: TetrisBoard,
    ticker_handle: Option<Timeout>,
    move_handle: (bool,Option<Timeout>),
    down_handle: Option<Timeout>,
    stick_handle: Option<Timeout>,
    level: u32,
    score: u32,
    lines_cleared: u32,
    stick_counter: u32,
    held_piece: Option<TetrisPieceType>,
    held_piece_switch_count: u32,
    piece_queue: VecDeque<TetrisPieceType>,
    settings: Settings
}

impl Component for GameDisplay {
    type Message = GameMsg;
    type Properties = ();

    fn create(ctx: &Context<Self>) -> Self {
        GameDisplay { game: TetrisBoard::make(10,20,TetrisPieceType::get_random()), ticker_handle: None, move_handle: (true,None), 
            down_handle: None, settings: Settings::default(), level: 1, stick_handle: None, stick_counter: 0, held_piece: None, held_piece_switch_count: 0,
            piece_queue: VecDeque::from_iter(Randomizers::RandomGenerator.make_sequence(7).into_iter()), score: 0, lines_cleared: 0}
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            GameMsg::Left(forced) => {
                if self.move_handle.1.is_none() || forced || self.move_handle.0 && self.move_handle.1.is_some(){
                    self.game.move_left();
                    let handle = if forced{
                        let link = _ctx.link().clone();
                        Timeout::new(self.settings.hold_move_interval, move || link.send_message(GameMsg::Left(true)))
                    } else {
                        let link = _ctx.link().clone();
                        Timeout::new(self.settings.hold_time, move || link.send_message(GameMsg::Left(true)))
                    };
                    self.move_handle = (false,Some(handle));
                    if self.stick_counter<10{
                        self.stick_handle=None;
                    }
                }
            }
            GameMsg::Right(forced) => {
                if self.move_handle.1.is_none() || forced || !self.move_handle.0 && self.move_handle.1.is_some(){
                    self.game.move_right();
                    let handle = if forced{
                        let link = _ctx.link().clone();
                        Timeout::new(self.settings.hold_move_interval, move || link.send_message(GameMsg::Right(true)))
                    } else {
                        let link = _ctx.link().clone();
                        Timeout::new(self.settings.hold_time, move || link.send_message(GameMsg::Right(true)))
                    };
                    self.move_handle = (true,Some(handle));
                    if self.stick_counter<10{
                        self.stick_handle=None;
                    }
                }
            }
            GameMsg::Down => {
                if self.game.move_down(){
                    self.score+=1;
                }
            }
            GameMsg::Drop => {
                self.score += self.game.drop()*2;
                if !self.game.new_falling_piece(self.piece_queue.pop_front().unwrap_or(TetrisPieceType::I)){
                    // reset game
                    self.game = TetrisBoard::make(10,20,TetrisPieceType::get_random());
                    self.ticker_handle=None;
                    self.level = 1;
                    self.score = 0;
                    self.lines_cleared=0;
                }
                if self.piece_queue.len()<=self.settings.queue_display_len{ self.piece_queue.extend(self.settings.randomizer.make_sequence(self.settings.queue_display_len))}
                self.piece_queue.push_back(TetrisPieceType::get_random());
                let num_cleared: u32 = self.game.clear_lines();
                self.score += [0,100,300,500,800][num_cleared as usize]*self.level;
                self.lines_cleared+=num_cleared;
                self.level=self.lines_cleared/10+1;
                self.game.update_drop_loc();
                self.stick_counter=0;
                self.stick_handle=None;
                self.held_piece_switch_count=0;
            }
            GameMsg::Tick => {
                if !self.game.move_down(){
                    self.stick_counter+=1;
                    if self.stick_handle.is_none(){
                        self.stick_handle = Some({
                            let link = _ctx.link().clone();
                            Timeout::new(self.get_tick_speed()*3, move || link.send_message(GameMsg::Drop))
                        }); 
                    }
                }
                let handle = {
                    let link = _ctx.link().clone();
                    Timeout::new(self.get_tick_speed(), move || link.send_message(GameMsg::Tick))
                };
                self.ticker_handle=Some(handle);
            }
            GameMsg::Hold => {
                self.held_piece_switch_count+=1;
                if self.held_piece_switch_count>self.settings.max_num_held_piece_switches{
                    return true
                }
                let curr_falling = self.game.falling_piece;
                if self.held_piece.is_none(){
                    self.game.new_falling_piece(TetrisPieceType::from_int(rand::thread_rng().gen_range(0..7)));
                }else{
                    self.game.new_falling_piece(self.held_piece.unwrap());
                }
                self.game.update_drop_loc();
                self.held_piece=Some(curr_falling);
            }
            GameMsg::Rotate => {
                self.game.rotate_clockwise();
                if self.stick_counter<10{
                    self.stick_handle=None;
                }
            }
            GameMsg::CancelDown => {

            }
            GameMsg::CancelLeft => {
                if !self.move_handle.0{self.move_handle=(true,None);}
            }
            GameMsg::CancelRight => {
                if self.move_handle.0{self.move_handle=(true,None);}
            }
            GameMsg::None => {

            }
        }
        true
    }
    fn view(&self, ctx: &Context<Self>) -> Html {
        let link = ctx.link();

        html!{
            <div class="game" onclick={link.callback(|_| GameMsg::Tick)} tabindex=0 onkeydown={link.callback(|key:KeyboardEvent| {match key.key_code(){67=>GameMsg::Hold,40=>GameMsg::Down, 39=>GameMsg::Right(false), 38=>GameMsg::Rotate, 37=>GameMsg::Left(false), 32 =>GameMsg::Drop,_=>GameMsg::None}})}
            onkeyup={link.callback(|key:KeyboardEvent| {match key.key_code(){40=>GameMsg::CancelDown, 39=>GameMsg::CancelRight, 37=>GameMsg::CancelLeft, _=>GameMsg::None}})}>
                // <h1>{"test"}</h1>
                // <button class="start-button" onclick={link.callback(|_| GameMsg::Tick)} onkeydown={link.callback(|key:KeyboardEvent| {match key.key_code(){67=>GameMsg::Hold,40=>GameMsg::Down, 39=>GameMsg::Right(false), 38=>GameMsg::Rotate, 37=>GameMsg::Left(false), 32 =>GameMsg::Drop,_=>GameMsg::None}})}
                // onkeyup={link.callback(|key:KeyboardEvent| {match key.key_code(){40=>GameMsg::CancelDown, 39=>GameMsg::CancelRight, 37=>GameMsg::CancelLeft, _=>GameMsg::None}})}/>
                <div class="inline-block">
                    {TetrisPieceType::view(&self.held_piece)}
                    <p>{self.score.to_string()}</p>
                </div>
                <div class="inline-block">
                    {self.game.view()}
                </div>
                <div class="inline-block">
                {
                    (0..self.settings.queue_display_len).map(|v|{
                        html!{TetrisPieceType::view(&Some(self.piece_queue[v].clone()))}
                    }).collect::<Html>()
                }
                </div>
            </div>
        }
    }
}
impl GameDisplay{
    fn get_tick_speed(&self) -> u32{
        return ((0.8-((self.level-1) as f32)*0.007).powf((self.level-1) as f32)*1000_f32) as u32
    }
}

struct Settings{
    hold_time: u32,
    hold_move_interval: u32,
    max_num_held_piece_switches: u32,
    queue_display_len: usize,
    randomizer: Randomizers
}
impl Default for Settings{
    fn default() -> Settings{
        Settings{hold_time: 120, hold_move_interval: 60, max_num_held_piece_switches: 1, queue_display_len: 4, randomizer: Randomizers::RandomGenerator}
    }
}

enum Randomizers{
    RandomGenerator,
    Random
}
impl Randomizers{
    fn make_sequence(&self, len: usize) -> Vec<TetrisPieceType>{
        match &self{
            Self::RandomGenerator => {
                let mut temp = (0..((len/7+1)*7)).map(|i| TetrisPieceType::from_int((i%7) as i32)).collect::<Vec<TetrisPieceType>>();
                for i in 0..temp.len(){
                    let swap_idx = rand::thread_rng().gen_range(0..7);
                    let tempt = temp[swap_idx];
                    temp[swap_idx]=temp[i].clone();
                    temp[i]=tempt;
                }
                temp
            }
            Self::Random => {
                Vec::from_iter((0..len).map(|_| TetrisPieceType::get_random()).collect::<Vec<TetrisPieceType>>())
            }
        }
    }
}

#[derive(Clone,Copy, PartialEq)]
enum TetrisPieceType{
    I,
    L,
    J,
    O,
    S,
    T,
    Z
}
impl TetrisPieceType{
    fn get_idx_arr(&self, rot: usize) -> [isize;4]{
        match rot%4{
            2 => match &self{
                Self::I => [4,5,6,7],
                Self::L =>  [0,4,5,6],
                Self::J => [2,4,5,6],
                Self::O => [1,2,5,6],
                Self::Z => [1,2,4,5],
                Self::T => [1,4,5,6],
                Self::S => [0,1,5,6],
            }
            1 => match &self{
                Self::I => [2,6,10,14],
                Self::L =>  [1,2,5,9],
                Self::J => [1,5,9,10],
                Self::O => [1,2,5,6],
                Self::S => [2,5,6,9],
                Self::T => [1,5,6,9],
                Self::Z => [1,5,6,10],
            }
            0 => match &self{
                Self::I => [8,9,10,11],
                Self::L =>  [4,5,6,10],
                Self::J => [4,5,6,8],
                Self::O => [1,2,5,6],
                Self::S => [4,5,9,10],
                Self::T => [4,5,6,9],
                Self::Z => [5,6,8,9],
            }
            3 => match &self{
                Self::I => [1,5,9,13],
                Self::L =>  [1,5,8,9],
                Self::J => [0,1,5,9],
                Self::O => [1,2,5,6],
                Self::S => [1,4,5,8],
                Self::T => [1,4,5,9],
                Self::Z => [0,4,5,9],
            }
            _ => [0,0,0,0]
        }
    }
    fn horiz_extents(&self, rot: usize) -> (isize,isize){
        match rot%4{
            2 => match &self{
                Self::I => (0,4),
                Self::L =>  (0,3),
                Self::J => (0,3),
                Self::O => (1,3),
                Self::Z => (0,3),
                Self::T => (0,3),
                Self::S => (0,3),
            }
            1 => match &self{
                Self::I => (2,3),
                Self::L =>  (1,3),
                Self::J => (1,3),
                Self::O => (1,3),
                Self::S => (1,3),
                Self::T => (1,3),
                Self::Z => (1,3),
            }
            0 => match &self{
                Self::I => (0,4),
                Self::L =>  (0,3),
                Self::J => (0,3),
                Self::O => (1,3),
                Self::S => (0,3),
                Self::T => (0,3),
                Self::Z => (0,3),
            }
            3 => match &self{
                Self::I => (1,2),
                Self::L =>  (0,2),
                Self::J => (0,2),
                Self::O => (1,3),
                Self::S => (0,2),
                Self::T => (0,2),
                Self::Z => (0,2),
            }
            _ => (0,0)
        }
    }
    fn vert_extents(&self, rot: usize) -> (isize,isize){
        match rot%4{
            1 => match &self{
                Self::I => (0,4),
                Self::L =>  (0,3),
                Self::J => (0,3),
                Self::O => (0,2),
                Self::Z => (0,3),
                Self::T => (0,3),
                Self::S => (0,3),
            }
            0 => match &self{
                Self::I => (2,3),
                Self::L =>  (1,3),
                Self::J => (1,3),
                Self::O => (0,2),
                Self::S => (1,3),
                Self::T => (1,3),
                Self::Z => (1,3),
            }
            3 => match &self{
                Self::I => (0,4),
                Self::L =>  (0,3),
                Self::J => (0,3),
                Self::O => (0,2),
                Self::S => (0,3),
                Self::T => (0,3),
                Self::Z => (0,3),
            }
            2 => match &self{
                Self::I => (1,2),
                Self::L =>  (0,2),
                Self::J => (0,2),
                Self::O => (0,2),
                Self::S => (0,2),
                Self::T => (0,2),
                Self::Z => (0,2),
            }
            _ => (0,0)
        }
    }
    fn secondary_tests(&self, rot: usize) -> Vec<(isize,isize)>{
        match rot%4{
            1 => match &self{
                Self::I => vec![(-2, 0),(1, 0),(-2,-1),(1,2)],
                Self::O => vec![],
                Self::J|Self::L|Self::S|Self::T|Self::Z => vec![(-1, 0),(-1,1),(0,-2),(-1,-2)]
            }
            0 => match &self{
                Self::I => vec![( 0, 0),(1, 0),(-2, 0),(1,-2),(-2,1)],
                Self::O => vec![],
                Self::J|Self::L|Self::S|Self::T|Self::Z => vec![( 0, 0),(-1, 0),(-1,-1),( 0,2),(-1,2)]
            }
            3 => match &self{
                Self::I => vec![( 0, 0),(2, 0),(-1, 0),(2,1),(-1,-2)],
                Self::O => vec![],
                Self::J|Self::L|Self::S|Self::T|Self::Z => vec![( 0, 0),(1, 0),(1,1),( 0,-2),(1,-2)]
            }
            2 => match &self{
                Self::I => vec![( 0, 0),(-1, 0),(2, 0),(-1,2),(2,-1)],
                Self::O => vec![],
                Self::J|Self::L|Self::S|Self::T|Self::Z => vec![( 0, 0),(1, 0),(1,-1),( 0,2),(1,2)]
            }
            _ => vec![]
        }
    }
    fn from_int(val: i32) -> Self{
        match val%7{
            0 => Self::I,
            1 => Self::J,
            2 => Self::L,
            3 => Self::O, 
            4 => Self::S,
            5 => Self::T,
            6 => Self::Z,
            _ => Self::T
        }
    }
    fn get_random() -> Self{
        TetrisPieceType::from_int(rand::thread_rng().gen_range(0..7))
    }
}

impl TetrisPieceType{
    fn view(from: &Option<Self>) -> Html{
        html!{
            <div class="piece-display">
                {
                    (0..4).rev().map(|r|{
                        html!{
                            {
                                (0..4).map(|c| {
                                    html!{
                                        if from.is_some() && from.unwrap().get_idx_arr(0).contains(&((c+r*4) as isize)){
                                            <span class="block"/>
                                        }else{
                                            <span class="empty-tile"/>
                                        }
                                    }
                                }).collect::<Html>()
                            }
                        }
                    }).collect::<Html>()
                }
            </div>
        }
    }
}

struct TetrisBoard{
    tiles: Vec<bool>,
    dimentions: (isize, isize),
    falling_piece: TetrisPieceType,
    falling_loc: isize,
    falling_rot: usize,
    drop_loc: isize
}

impl TetrisBoard{
    fn make(width: usize, height: usize, first_falling_piece: TetrisPieceType) -> Self{
        let mut tiles = vec![false;width*(height+3)];
        // tiles[0]=true;
        Self{tiles, dimentions: (width as isize, (height+3) as isize), falling_piece:first_falling_piece, 
        falling_loc: 193, falling_rot:0, drop_loc: 3}
    }
    
    fn check_loc_for_falling_piece(&self, idx: isize) -> bool{
        for i in self.falling_piece.get_idx_arr(self.falling_rot){
            if self.falling_loc<self.dimentions.0*self.dimentions.1 && self.falling_loc+i/4*self.dimentions.0+i%4==idx{ return true }
        }
        return false
    }
    fn check_drop_loc(&self, idx: isize) -> bool{
        for i in self.falling_piece.get_idx_arr(self.falling_rot){
            if self.drop_loc<self.dimentions.0*self.dimentions.1 && self.drop_loc+i/4*self.dimentions.0+i%4==idx{ return true }
        }
        return false
    }
    fn move_down(&mut self) -> bool{
        // if self.falling_loc>=self.dimentions.0{
        //     self.falling_loc-=self.dimentions.0;
        //     if self.check_overlap(){
        //         self.falling_loc+=self.dimentions.0;
        //     }
        // }
        self.falling_loc-=self.dimentions.0;
        if !self.check_in_bounds() || self.check_overlap(){
            self.falling_loc+=self.dimentions.0;
            return false
        }
        return true
    }
    fn move_right(&mut self){
        // if self.falling_loc%self.dimentions.0!=(self.dimentions.0-self.falling_piece.horiz_extents(self.falling_rot).1)%self.dimentions.0{
        //     self.falling_loc+=1;
        //     if self.check_overlap(){
        //         self.falling_loc-=1;
        //     }
        // }
        if !(self.falling_piece==TetrisPieceType::I&&(self.falling_loc+self.falling_piece.horiz_extents(self.falling_rot).0)%self.dimentions.0==9){
            self.falling_loc+=1;
        }
        if !self.check_in_bounds() || self.check_overlap(){
            self.falling_loc-=1;
        }else{
            self.update_drop_loc();
        }
    }
    fn move_left(&mut self){
        /*
        if (self.falling_loc%self.dimentions.0) != (self.dimentions.0-self.falling_piece.horiz_extents(self.falling_rot).0)%self.dimentions.0{
            self.falling_loc-=1;
            if self.check_overlap(){
                self.falling_loc+=1;
            }
        }*/
        if !(self.falling_piece==TetrisPieceType::I&&(self.falling_loc+self.falling_piece.horiz_extents(self.falling_rot).0)%self.dimentions.0==0){
            self.falling_loc-=1;
        }
        if !self.check_in_bounds() || self.check_overlap(){
            self.falling_loc+=1;
        }else{
            self.update_drop_loc();
        }
    }
    fn clear_lines(&mut self) -> u32{
        let mut line_counter = 0;
        for r in 0..self.dimentions.1{
            let mut filled: bool = true;
            for c in 0..self.dimentions.0{
                if self.tiles[(r*self.dimentions.0+c) as usize] == false{
                    filled=false;
                }
            }
            if filled{
                line_counter+=1;
                for c in 0..self.dimentions.0{
                    self.tiles[(r*self.dimentions.0+c) as usize]=false;
                }
            }else{
                for c in 0..self.dimentions.0{
                    self.tiles[((r-line_counter)*self.dimentions.0+c) as usize]=self.tiles[(r*self.dimentions.0+c) as usize];
                }
            }
        }
        return line_counter as u32
    }
    fn update_drop_loc(&mut self) -> isize{
        let mut i = 0;
        while self.move_down(){
            i+=1;
        }
        self.drop_loc=self.falling_loc;
        self.falling_loc+=i*self.dimentions.0;
        return self.drop_loc
    }
    fn rotate_clockwise(&mut self){
        self.falling_rot=(self.falling_rot+1)%4;
        if !self.check_in_bounds() || self.check_overlap(){
            for (x,y) in self.falling_piece.secondary_tests(self.falling_rot){
                let d = x+y*self.dimentions.0;
                self.falling_loc+=d;
                if self.check_in_bounds() && !self.check_overlap(){
                    self.update_drop_loc();
                    return
                }
                self.falling_loc-=d;
            }
            self.falling_rot=(self.falling_rot+3)%4;
        }
        self.update_drop_loc();
    }
    fn check_in_bounds(&self) -> bool{
        let horiz_extents = self.falling_piece.horiz_extents(self.falling_rot);
        return (self.falling_loc+2*self.dimentions.0+horiz_extents.0)%self.dimentions.0<=self.dimentions.0-horiz_extents.1+horiz_extents.0 &&
        self.falling_piece.vert_extents(self.falling_rot).0+self.row()>=0
    }
    fn row(&self) -> isize{
        return if self.falling_loc+self.falling_piece.horiz_extents(self.falling_rot).0<0 {(self.falling_loc+1-self.dimentions.0)/self.dimentions.0} else {self.falling_loc/self.dimentions.0}
    }
    fn check_overlap(&self) -> bool{
        for i in self.falling_piece.get_idx_arr(self.falling_rot){
            let loc: isize = self.falling_loc+i/4*self.dimentions.0+i%4;
            if loc>=0 && loc<self.dimentions.0*self.dimentions.1 && self.tiles[loc as usize]==true{
                return true
            }
        }
        return false 
    }
    fn drop(&mut self) -> u32{
        let mut cells_dropped = 0;
        while self.move_down(){cells_dropped+=1};
        for i in self.falling_piece.get_idx_arr(self.falling_rot){
            let loc: isize = self.falling_loc+i/4*self.dimentions.0+i%4;
            if loc>=0 && loc<self.dimentions.0*self.dimentions.1{
                self.tiles[loc as usize]=true;
            }
        }
        // self.new_falling_piece();
        cells_dropped
    }
    fn new_falling_piece(&mut self, new_piece: TetrisPieceType) -> bool{
        self.falling_piece=new_piece;
        self.falling_loc=193;
        self.falling_rot=0;
        if self.check_overlap(){ self.falling_loc+=self.dimentions.0; }
        !self.check_overlap()
    }
}

#[derive(Clone, PartialEq, Properties)]
struct TetrisBoardProps{
    width: usize,
    height: usize,
}

impl TetrisBoard{
    fn view(&self) -> Html {
        html!{
            <div class="board">
                {
                    (0..(self.dimentions.1-3)).rev().map(|r|{
                        html!{
                            {
                                (0..self.dimentions.0).map(|c| {
                                    html!{
                                        if self.tiles[(c+r*self.dimentions.0) as usize]{
                                            <span class="block"/>
                                        }else if self.check_loc_for_falling_piece(c+r*self.dimentions.0){
                                            <span class="block"/>
                                        }else if self.check_drop_loc(c+r*self.dimentions.0){
                                            <span class="block translucent"/>
                                        }else{
                                            <span class="empty-tile"/>
                                        }
                                    }
                                }).collect::<Html>()
                            }
                        }
                    }).collect::<Html>()
                    /*
                    self.tiles.iter().rev().enumerate().map(|(i,b)|{
                        html!{
                            if *b{
                                <span class="block"/>
                            }else if self.check_loc_for_falling_piece(i){
                                <span class="block"/>
                            }else{
                                <span class="empty-tile"/>
                            }
                        }
                    }).collect::<Html>()*/
                }
            </div>
        }
    }
}
