use std::{collections::VecDeque, io::BufRead, fmt::{Display, Formatter, Result}, clone};

use yew::prelude::*;
use web_sys::{HtmlInputElement, TouchEvent, TouchList, Touch, Element, Event, window, HtmlDocument};
use wasm_bindgen::{JsCast, JsValue, UnwrapThrowExt};
use gloo::{console::{self, Timer, dirxml}, timers::callback, events::EventListener, utils::document};
use gloo::timers::callback::{Interval, Timeout};
use rand::Rng;

fn main() {
    println!("Hello, world!");
    yew::start_app::<RootComponent>();
}

enum SettingsMsg{
    ToggleSettingsWindow,
    ChangeColor(String, usize),
    ChangeSettings(String, u32),
    SetTheme(u32),
    SaveCookies,
}

struct RootComponent{
    game_settings: Settings,
    displaying_window: bool,
    colors: Vec<String>,
    cookies: String
}
impl Component for RootComponent{
    type Message = SettingsMsg;

    type Properties = ();

    fn create(ctx: &Context<Self>) -> Self {
        let get_cookies = document().unchecked_into::<HtmlDocument>().cookie().unwrap_or(String::from("None"));
        let mut colors: Vec<String> = vec![String::from("#2c2a29"),String::from("#333333"),String::from("#222222"),String::from("#a7a7a7"),String::from("#ffffff"),String::from("#00ffff"),String::from("#ffff00")
        ,String::from("#ff00ff"),String::from("#ffa500"),String::from("#0000ff"),String::from("#ff0000"),String::from("#00ff00")];
        let mut game_settings = Settings::default();
        if get_cookies!="None"{
            // colors = Vec::new();
            for v in get_cookies.split("; "){
                match v.split_once('='){
                    Some((name, value)) => {
                        if name.starts_with("saved_color_"){
                            colors[name.split_at(12).1.parse::<usize>().unwrap_or(0)] = String::from(value);
                        }else{
                            match name{
                                "hold_time" => game_settings.hold_time=value.parse::<u32>().unwrap_or(game_settings.hold_time),
                                "hold_move_interval" => game_settings.hold_move_interval=value.parse::<u32>().unwrap_or(game_settings.hold_move_interval),
                                "max_switches" => game_settings.max_num_held_piece_switches=value.parse::<u32>().unwrap_or(1),
                                "randomizer" => game_settings.randomizer=match value {"Random" => Randomizers::Random, _ => Randomizers::RandomGenerator},
                                "lock_delay" => game_settings.lock_delay=value.parse::<u32>().unwrap_or(500),
                                "moves_before_lock" => game_settings.moves_before_lock=value.parse::<u32>().unwrap_or(15),
                                _ => {}
                            }
                        }
                    }
                    None => {
                        // this cookie does not exist. Error? idk
                    }
                }
            }
        }
        Self{game_settings: Settings::default(), displaying_window: false, colors, cookies: get_cookies}
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg{
            SettingsMsg::ToggleSettingsWindow => {
                self.displaying_window = !self.displaying_window;
            }
            SettingsMsg::ChangeColor(color, id) => {
                self.colors[id]=color;
            }
            SettingsMsg::ChangeSettings(value, id) => {
                match id{
                    0 => {
                        self.game_settings.hold_time=value.parse::<u32>().unwrap_or(self.game_settings.hold_time);
                    }
                    1 => {
                        self.game_settings.hold_move_interval=value.parse::<u32>().unwrap_or(self.game_settings.hold_move_interval);
                    }
                    2 => {
                        self.game_settings.max_num_held_piece_switches=value.parse::<u32>().unwrap_or(self.game_settings.max_num_held_piece_switches);
                    }
                    3 => {
                        self.game_settings.queue_display_len=value.parse::<usize>().unwrap_or(self.game_settings.queue_display_len);
                    }
                    4 => {
                        self.game_settings.randomizer=match self.game_settings.randomizer{Randomizers::Random => Randomizers::RandomGenerator, _ => Randomizers::Random}
                    }
                    5 => {
                        self.game_settings.lock_delay=value.parse::<u32>().unwrap_or(500)
                    }
                    6 => {
                        self.game_settings.moves_before_lock=value.parse::<u32>().unwrap_or(15)
                    }
                    _ => {

                    }
                }
            }
            SettingsMsg::SetTheme(theme_id) => {
                match theme_id {
                    1 => {
                        self.colors = vec![String::from("#2c2a29"),String::from("#353231"),String::from("#222120"),String::from("#cfb24a"),String::from("#ffffff"),String::from("#00ffff"),String::from("#ffff00")
                        ,String::from("#ff00ff"),String::from("#ffa500"),String::from("#0000ff"),String::from("#ff0000"),String::from("#00ff00")];
                        self.game_settings = Settings::default();
                    }
                    _ => {
                        self.colors = vec![String::from("#2c2a29"),String::from("#333333"),String::from("#222222"),String::from("#a7a7a7"),String::from("#ffffff"),String::from("#00ffff"),String::from("#ffff00")
                        ,String::from("#ff00ff"),String::from("#ffa500"),String::from("#0000ff"),String::from("#ff0000"),String::from("#00ff00")];
                        self.game_settings = Settings::default();
                    }
                }
            }
            SettingsMsg::SaveCookies => {
                let doc = document().unchecked_into::<HtmlDocument>();
                for i in 0..self.colors.len(){
                    let _ = doc.set_cookie(&format!("saved_color_{}={}",i,self.colors[i]));
                }
                let _ = doc.set_cookie(&format!("hold_time={}",self.game_settings.hold_time));
                let _ = doc.set_cookie(&format!("hold_move_interval={}",self.game_settings.hold_move_interval));
                let _ = doc.set_cookie(&format!("max_switches={}",self.game_settings.max_num_held_piece_switches));
                let _ = doc.set_cookie(&format!("lock_delay={}",self.game_settings.lock_delay));
                let _ = doc.set_cookie(&format!("moves_before_lock={}",self.game_settings.moves_before_lock));
                let _ = doc.set_cookie(&format!("randomizer={}",self.game_settings.randomizer.to_string()));
            }
        }
        true
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let doc = document().unchecked_into::<HtmlDocument>();
        let r = doc.query_selector("html, body").unwrap().unwrap();
        let _ = r.set_attribute("style", &format!("background-color: {};",self.colors[0]));
        let link = ctx.link();
        html!{
            <div class="root" style={format!("--bg-color: {}; --board-bg: {}; --board-outline: {}; --text-color: {}; --accent-target: {}; --Icolor: {}; --Ocolor: {}; --Tcolor:{}; --Lcolor: {}; --Jcolor: {};
            --Scolor: {}; --Zcolor: {};",self.colors[0],self.colors[1], self.colors[2],self.colors[3],self.colors[4],self.colors[5],self.colors[6],
            self.colors[7],self.colors[8],self.colors[9],self.colors[10],self.colors[11])}>
                <h1>{"Testris"}</h1>
                <p>{self.colors[0].clone()}</p>
                <button class="settings" onclick={link.callback(|_| SettingsMsg::ToggleSettingsWindow)}>
                {"settings"}
                </button>
                <hr/>
                if self.displaying_window{
                    <div class="settings-window">
                    <input type="color" value={self.colors[0].clone()} onchange={Self::get_color_callback(link,0)}/>
                    <input type="color" value={self.colors[1].clone()} onchange={Self::get_color_callback(link,1)}/>
                    <input type="color" value={self.colors[2].clone()} onchange={Self::get_color_callback(link,2)}/>
                    <input type="color" value={self.colors[3].clone()} onchange={Self::get_color_callback(link,3)}/>
                    <input type="color" value={self.colors[4].clone()} onchange={Self::get_color_callback(link,4)}/>
                    <input type="color" value={self.colors[5].clone()} onchange={Self::get_color_callback(link,5)}/>
                    <input type="color" value={self.colors[6].clone()} onchange={Self::get_color_callback(link,6)}/>
                    <input type="color" value={self.colors[7].clone()} onchange={Self::get_color_callback(link,7)}/>
                    <input type="color" value={self.colors[8].clone()} onchange={Self::get_color_callback(link,8)}/>
                    <input type="color" value={self.colors[9].clone()} onchange={Self::get_color_callback(link,9)}/>
                    <input type="color" value={self.colors[10].clone()} onchange={Self::get_color_callback(link,10)}/>
                    <input type="color" value={self.colors[11].clone()} onchange={Self::get_color_callback(link,11)}/>
                    <div class="horiz-section">
                    <h1>{"key held delay"}</h1>
                    <div class="text">{"The amount of time in milliseconds that a left/right must be held before the piece moves sideways"}</div>
                    <input name="key-held-delay" type="number" value={self.game_settings.hold_time.to_string()} onchange={Self::get_settings_callback(link,0)}/>
                    </div>
                    <div class="horiz-section">
                    <h1>{"key held speed"}</h1>
                    <div class="text">{"The time in milliseconds between sideways piece movements when left/right is being held. (changes how fast pieces move side to side when keys are held)"}</div>
                    <input name="key-held-speed" type="number" value={self.game_settings.hold_move_interval.to_string()} onchange={Self::get_settings_callback(link,1)}/>
                    </div>
                    <div class="horiz-section">
                    <h1>{"held piece switches"}</h1>
                    <div class="text">{"The number of times you can switch between active and held piece"}</div>
                    <input name="max-switches-for-held-piece" type="number" min="0" value={self.game_settings.max_num_held_piece_switches.to_string()} onchange={Self::get_settings_callback(link,2)}/>
                    </div>
                    <div class="horiz-section">
                    <h1>{"queue display length"}</h1>
                    <div class="text">{"Length of the next pieces display"}</div>
                    <input name="queue-display-len" type="range" min=0 max=6 value={self.game_settings.queue_display_len.to_string()} onchange={Self::get_settings_callback(link,3)}/>
                    <h2>{self.game_settings.queue_display_len}</h2>
                    </div>
                    <div class="horiz-section">
                    <h1>{"lock delay"}</h1>
                    <div class="text">{"Time in milliseconds until a piece \"locks\" into place"}</div>
                    <input name="lock-delay" type="number" value={self.game_settings.lock_delay.to_string()} onchange={Self::get_settings_callback(link,5)}/>
                    </div>
                    <div class="horiz-section">
                    <h1>{"moves before lock"}</h1>
                    <div class="text">{"Number of inputs that reset the \"lock delay\" before the piece is forcibly locked into place"}</div>
                    <input name="moves-before-lock" type="number" value={self.game_settings.moves_before_lock.to_string()} onchange={Self::get_settings_callback(link,6)}/>
                    </div>
                    <div class="horiz-section">
                    <h1>{"randomizer"}</h1>
                    <div class="text">{"Which randomizer algorithmn to use for generating next pieces (random = fully random, randomgenerator = randomly sorts 7 pieces at a time)"}</div>
                    <button onclick={link.callback(|_| SettingsMsg::ChangeSettings(String::new(),4))}>{self.game_settings.randomizer.to_string()}</button>
                    </div>
                    <div class="horiz-section">
                        <h1>{"theme"}</h1>
                        <div class="text">{"Toggles between themes"}</div>
                        <button onclick={link.callback(|_| SettingsMsg::SetTheme(0))}>{"Toggle Theme"}</button>
                    </div>

                    <button onclick={link.callback(|_| SettingsMsg::SaveCookies)}>
                    {"save"}
                    </button>
                    <p>{self.cookies.clone()}</p>
                    </div>
                }
                // <div class="notouch"></div>
                else{
                <GameDisplay settings={self.game_settings.clone()}/>
                }
            </div>
        }
    }
}
impl RootComponent{
    fn get_color_callback(link: &yew::html::Scope<Self>, val: usize) -> yew::Callback<Event>{
        return {link.callback(move |e: Event| {let input: HtmlInputElement = e.target_unchecked_into(); SettingsMsg::ChangeColor(input.value().parse::<String>().unwrap(),val)})}
    }
    fn get_settings_callback(link: &yew::html::Scope<Self>, val: u32) -> yew::Callback<Event>{
        return {link.callback(move |e: Event| {let input: HtmlInputElement = e.target_unchecked_into(); SettingsMsg::ChangeSettings(input.value().parse::<String>().unwrap(),val)})}
    }
}

#[derive(Properties,PartialEq, Clone)]
struct GameProps{
    settings: Settings
}

enum GameMsg {
    Left(InputTypes),
    Right(InputTypes),
    Down,
    Drop,
    Tick,
    Hold,
    Rotate,
    CancelDown,
    CancelRight,
    CancelLeft,
    TouchStart(TouchEvent),
    TouchMove(TouchEvent),
    TouchEnd(TouchEvent),
    Unfocus,
    None
}

#[derive(PartialEq)]
enum InputTypes{
    Tap,
    Hold,
    Touch
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
    touch_start_pos: (i32,i32),
    touch_translation: i32,
    touch_pos: (i32,i32),
    touch_velocity: (i32,i32),
    touch_can_rotate: bool,
    settings: Settings
}

impl Component for GameDisplay {
    type Message = GameMsg;
    type Properties = GameProps;

    fn create(ctx: &Context<Self>) -> Self {
        let mut piece_queue: VecDeque<TetrisPieceType> = VecDeque::from_iter(ctx.props().settings.randomizer.make_sequence(7).into_iter());
        let first_piece = piece_queue.pop_front().unwrap_or(TetrisPieceType::I);
        GameDisplay { game: TetrisBoard::make(10,20,first_piece), ticker_handle: None, move_handle: (true,None), 
            down_handle: None, settings: ctx.props().settings.clone(), level: 1, stick_handle: None, stick_counter: 0, held_piece: None, held_piece_switch_count: 0,
            piece_queue, score: 0, lines_cleared: 0,
            touch_start_pos: (0,0), touch_pos: (0,0), touch_translation: 0, touch_velocity: (0,0), touch_can_rotate: true}
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            GameMsg::Left(t) => {
                if self.move_handle.1.is_none() || t==InputTypes::Hold || self.move_handle.0 && self.move_handle.1.is_some(){
                    self.game.move_left();
                    if t!=InputTypes::Touch{
                        let handle = if t==InputTypes::Hold{
                            let link = _ctx.link().clone();
                            Timeout::new(self.settings.hold_move_interval, move || link.send_message(GameMsg::Left(InputTypes::Hold)))
                        } else {
                            let link = _ctx.link().clone();
                            Timeout::new(self.settings.hold_time, move || link.send_message(GameMsg::Left(InputTypes::Hold)))
                        };
                        self.move_handle = (false,Some(handle));
                    }
                    if self.stick_counter<self.settings.moves_before_lock{
                        self.stick_handle=None;
                    }
                }
            }
            GameMsg::Right(t) => {
                if self.move_handle.1.is_none() || t==InputTypes::Hold || !self.move_handle.0 && self.move_handle.1.is_some(){
                    self.game.move_right();
                    if t!=InputTypes::Touch{
                        let handle = if t==InputTypes::Hold{
                            let link = _ctx.link().clone();
                            Timeout::new(self.settings.hold_move_interval, move || link.send_message(GameMsg::Right(InputTypes::Hold)))
                        } else {
                            let link = _ctx.link().clone();
                            Timeout::new(self.settings.hold_time, move || link.send_message(GameMsg::Right(InputTypes::Hold)))
                        };
                        self.move_handle = (true,Some(handle));

                    }
                    if self.stick_counter<self.settings.moves_before_lock{
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
                    self.held_piece=None;
                }
                if self.piece_queue.len()<=self.settings.queue_display_len{ self.piece_queue.extend(self.settings.randomizer.make_sequence(self.settings.queue_display_len))}
                // self.piece_queue.push_back(TetrisPieceType::get_random());
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
                            Timeout::new(self.settings.lock_delay, move || link.send_message(GameMsg::Drop))
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
                    self.game.new_falling_piece(self.piece_queue.pop_front().unwrap_or(TetrisPieceType::I));
                }else{
                    self.game.new_falling_piece(self.held_piece.unwrap());
                }
                self.game.update_drop_loc();
                self.held_piece=Some(curr_falling);
                self.stick_counter=0;
                self.stick_handle=None;
            }
            GameMsg::Rotate => {
                self.game.rotate_clockwise();
                if self.stick_counter<self.settings.moves_before_lock{
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
            GameMsg::TouchStart(t) => {
                //t.prevent_default();
                _ctx.link().send_message(GameMsg::Tick);
                if self.ticker_handle.is_none(){
                    _ctx.link().send_message(GameMsg::Tick);
                    // {
                    //     let link = _ctx.link().clone();
                    //     Timeout::new(0, move || link.send_message(GameMsg::Tick))
                    // }.forget();
                }
                let first_touch = t.touches().get(0).unwrap();
                self.touch_start_pos=(first_touch.client_x(),first_touch.client_y());
                self.touch_pos=self.touch_start_pos;
                self.touch_translation=self.touch_start_pos.0;
            }
            GameMsg::TouchMove(t) => {
                //t.prevent_default();
                let first_touch = t.touches().get(0).unwrap();
                let pos=(first_touch.client_x(),first_touch.client_y());
                if pos.0-self.touch_translation>=25 && (pos.1-self.touch_start_pos.1).abs()<50{
                    self.touch_translation=pos.0;
                    self.touch_can_rotate=false;
                    _ctx.link().send_message(GameMsg::Right(InputTypes::Touch));
                    // let handle = {
                    //     let link = _ctx.link().clone();
                    //     Timeout::new(0, move || link.send_message(GameMsg::Right(InputTypes::Touch)))
                    // }.forget();
                }else if pos.0-self.touch_translation<=-25 && (pos.1-self.touch_start_pos.1).abs()<50{
                    self.touch_translation=pos.0;
                    self.touch_can_rotate=false;
                    _ctx.link().send_message(GameMsg::Left(InputTypes::Touch));
                    // let handle = {
                    //     let link = _ctx.link().clone();
                    //     Timeout::new(0, move || link.send_message(GameMsg::Left(InputTypes::Touch)))
                    // }.forget();
                }
                if pos.1-self.touch_start_pos.1>80{
                    self.touch_can_rotate=false;
                    _ctx.link().send_message(GameMsg::Down);
                    // let handle = {
                    //     let link = _ctx.link().clone();
                    //     Timeout::new(0, move || link.send_message(GameMsg::Down))
                    // }.forget();
                }
                self.touch_pos=pos;
            }
            GameMsg::TouchEnd(t) => {
                t.prevent_default();
                if self.touch_pos.1-self.touch_start_pos.1>40{
                    _ctx.link().send_message(GameMsg::Drop);
                    // let handle = {
                    //     let link = _ctx.link().clone();
                    //     Timeout::new(0, move || link.send_message(GameMsg::Drop))
                    // }.forget();
                }else if self.touch_start_pos.1-self.touch_pos.1>40{
                    _ctx.link().send_message(GameMsg::Hold);
                    // let handle = {
                    //     let link = _ctx.link().clone();
                    //     Timeout::new(0, move || link.send_message(GameMsg::Hold))
                    // }.forget();
                }else if self.touch_can_rotate && (self.touch_pos.0-self.touch_start_pos.0).abs() < 5{
                    _ctx.link().send_message(GameMsg::Rotate);
                    // let handle = {
                    //     let link = _ctx.link().clone();
                    //     Timeout::new(0, move || link.send_message(GameMsg::Rotate))
                    // }.forget();
                }
                self.touch_can_rotate=true;
            }
            GameMsg::Unfocus => {
                self.ticker_handle=None;
            }
            GameMsg::None => {

            }
        }
        true
    }
    fn view(&self, ctx: &Context<Self>) -> Html {
        let link = ctx.link();
        let dim = match window(){
            Some(w) => (w.outer_width().unwrap().as_f64().unwrap(),w.outer_height().unwrap().as_f64().unwrap()),
            None => (0.0,0.0)
        };
        html!{
            <div class="game no-touch-move" tabindex=0 onkeydown={link.callback(|key:KeyboardEvent| {match key.key_code(){67=>GameMsg::Hold,40=>GameMsg::Down, 39=>GameMsg::Right(InputTypes::Tap), 38=>GameMsg::Rotate, 37=>GameMsg::Left(InputTypes::Tap), 32 =>GameMsg::Drop,_=>GameMsg::None}})}
            onkeyup={link.callback(|key:KeyboardEvent| {match key.key_code(){40=>GameMsg::CancelDown, 39=>GameMsg::CancelRight, 37=>GameMsg::CancelLeft, _=>GameMsg::None}})}
            ontouchstart={link.callback(|t:TouchEvent| GameMsg::TouchStart(t))} ontouchmove={link.callback(|t| GameMsg::TouchMove(t))} ontouchend={link.callback(|t| GameMsg::TouchEnd(t))}
            onfocusin={link.callback(|_| GameMsg::Tick)} onfocusout={link.callback(|_| GameMsg::Unfocus)}>

                // <div class="notouch" tabindex=0 onkeydown={link.callback(|key:KeyboardEvent| {match key.key_code(){67=>GameMsg::Hold,40=>GameMsg::Down, 39=>GameMsg::Right(InputTypes::Tap), 38=>GameMsg::Rotate, 37=>GameMsg::Left(InputTypes::Tap), 32 =>GameMsg::Drop,_=>GameMsg::None}})}
                // onkeyup={link.callback(|key:KeyboardEvent| {match key.key_code(){40=>GameMsg::CancelDown, 39=>GameMsg::CancelRight, 37=>GameMsg::CancelLeft, _=>GameMsg::None}})}
                // ontouchstart={link.callback(|t:TouchEvent| GameMsg::TouchStart(t))} ontouchmove={link.callback(|t| GameMsg::TouchMove(t))} ontouchend={link.callback(|t| GameMsg::TouchEnd(t))}></div>
                // <h1>{"test"}</h1>
                // <button class="start-button" onclick={link.callback(|_| GameMsg::Tick)} onkeydown={link.callback(|key:KeyboardEvent| {match key.key_code(){67=>GameMsg::Hold,40=>GameMsg::Down, 39=>GameMsg::Right(false), 38=>GameMsg::Rotate, 37=>GameMsg::Left(false), 32 =>GameMsg::Drop,_=>GameMsg::None}})}
                // onkeyup={link.callback(|key:KeyboardEvent| {match key.key_code(){40=>GameMsg::CancelDown, 39=>GameMsg::CancelRight, 37=>GameMsg::CancelLeft, _=>GameMsg::None}})}/>
                <div class="inline-block">
                    {TetrisPieceType::view(&self.held_piece)}
                    <div class="sidebar-num-display">
                    <h1>{"Score"}</h1>
                    <p>{self.score.to_string()}</p>
                    </div>
                    <div class="sidebar-num-display">
                    <h1>{"Level"}</h1>
                    <p>{self.level.to_string()}</p>
                    </div>
                    // <p>{format!("start: {},{}",self.touch_start_pos.0,self.touch_start_pos.1)}</p>
                    // <p>{format!("curr: {},{}",self.touch_pos.0,self.touch_pos.1)}</p>
                    // <p>{format!("{}",self.piece_queue.len())}</p>
                    // <p>{format!("w{},h{}",dim.0,dim.1)}</p>
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

#[derive(PartialEq, Clone)]
struct Settings{
    hold_time: u32,
    hold_move_interval: u32,
    max_num_held_piece_switches: u32,
    queue_display_len: usize,
    lock_delay: u32,
    moves_before_lock: u32,
    randomizer: Randomizers
}
impl Default for Settings{
    fn default() -> Settings{
        Settings{hold_time: 150, hold_move_interval: 60, max_num_held_piece_switches: 1, queue_display_len: 4, lock_delay: 500, moves_before_lock: 15, randomizer: Randomizers::RandomGenerator}
    }
}

#[derive(PartialEq, Clone)]
enum Randomizers{
    RandomGenerator,
    Random
}
impl Randomizers{
    fn make_sequence(&self, len: usize) -> Vec<TetrisPieceType>{
        match &self{
            Self::RandomGenerator => {
                let mut temp = (0..(((len-1)/7+1)*7)).map(|i| TetrisPieceType::from_int((i%7) as i32)).collect::<Vec<TetrisPieceType>>();
                for i in 0..temp.len(){
                    let swap_idx = rand::thread_rng().gen_range(0..7)+(i/7)*7;
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
impl ToString for Randomizers{
    fn to_string(&self) -> String {
        match &self{
            Self::RandomGenerator => String::from("RandomGenerator"),
            Self::Random => String::from("Random")
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
                Self::L|Self::J|Self::Z|Self::T|Self::S =>  (0,3),
                Self::O => (1,3),
            }
            1 => match &self{
                Self::I => (2,3),
                Self::L|Self::J|Self::O|Self::S|Self::T|Self::Z =>  (1,3),
            }
            0 => match &self{
                Self::I => (0,4),
                Self::L|Self::J|Self::S|Self::T|Self::Z =>  (0,3),
                Self::O => (1,3),
            }
            3 => match &self{
                Self::I => (1,2),
                Self::L|Self::J|Self::S|Self::T|Self::Z =>  (0,2),
                Self::O => (1,3),
            }
            _ => (0,0)
        }
    }
    fn vert_extents(&self, rot: usize) -> (isize,isize){
        match rot%4{
            1 => match &self{
                Self::I => (0,4),
                Self::L|Self::J|Self::S|Self::T|Self::Z =>  (0,3),
                Self::O => (0,2),
            }
            0 => match &self{
                Self::I => (2,3),
                Self::L|Self::J|Self::S|Self::T|Self::Z =>  (1,3),
                Self::O => (0,2),
            }
            3 => match &self{
                Self::I => (0,4),
                Self::L|Self::J|Self::S|Self::T|Self::Z =>  (0,3),
                Self::O => (0,2),
            }
            2 => match &self{
                Self::I => (1,2),
                Self::L|Self::J|Self::S|Self::T|Self::Z|Self::O =>  (0,2),
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
                                            <span class={format!("sidebar-tile filled {}-color",from.unwrap())}/>
                                        }else{
                                            <span class="sidebar-tile empty"/>
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
impl Display for TetrisPieceType{
    fn fmt(&self, f: &mut Formatter) -> Result {
        write!(f,"{}",String::from(match self{
            Self::I => "I",
            Self::J => "J",
            Self::L => "L",
            Self::O => "O",
            Self::S => "S",
            Self::T => "T",
            Self::Z => "Z"
        }))
    }
}

struct TetrisBoard{
    tiles: Vec<Option<TetrisPieceType>>,
    dimentions: (isize, isize),
    falling_piece: TetrisPieceType,
    falling_loc: isize,
    falling_rot: usize,
    drop_loc: isize
}

impl TetrisBoard{
    fn make(width: usize, height: usize, first_falling_piece: TetrisPieceType) -> Self{
        let mut tiles = vec![None;width*(height+3)];
        // tiles[0]=true;
        Self{tiles, dimentions: (width as isize, (height+3) as isize), falling_piece:first_falling_piece, 
        falling_loc: 193, falling_rot:0, drop_loc: -7}
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
                if self.tiles[(r*self.dimentions.0+c) as usize].is_none(){
                    filled=false;
                }
            }
            if filled{
                line_counter+=1;
                for c in 0..self.dimentions.0{
                    self.tiles[(r*self.dimentions.0+c) as usize]=None;
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
            if loc>=0 && loc<self.dimentions.0*self.dimentions.1 && self.tiles[loc as usize].is_some(){
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
                self.tiles[loc as usize]=Some(self.falling_piece);
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
                                        if self.tiles[(c+r*self.dimentions.0) as usize].is_some(){
                                            <span class={format!("tile filled {}-color",self.tiles[(c+r*self.dimentions.0) as usize].unwrap())}/>
                                        }else if self.check_loc_for_falling_piece(c+r*self.dimentions.0){
                                            <span class={format!("tile filled {}-color",self.falling_piece)}/>
                                        }else if self.check_drop_loc(c+r*self.dimentions.0){
                                            <span class={format!("tile outline {}-color",self.falling_piece)}/>
                                        }else{
                                            <span class="tile empty"/>
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
