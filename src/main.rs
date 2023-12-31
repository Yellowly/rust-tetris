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
    ChangeWindow(Windows),
    ChangeColor(String, usize),
    ChangeSettings(String, u32),
    SetTheme(u32),
    Revert,
    SaveCookies(bool),
}

#[derive(PartialEq)]
enum Windows{
    Game,
    Settings,
    Highscores
}

struct RootComponent{
    game_settings: Settings,
    displaying_window: Windows,
    colors: Vec<String>,
    highscores: Vec<u32>,
    cookie_notif: bool,
    cookies: String
}
impl Component for RootComponent{
    type Message = SettingsMsg;

    type Properties = ();

    fn create(ctx: &Context<Self>) -> Self {
        let get_cookies = document().unchecked_into::<HtmlDocument>().cookie().unwrap_or(String::from("None"));
        let mut colors: Vec<String> = vec![String::from("#2c2a29"),String::from("#333333"),String::from("#222222"),String::from("#a7a7a7"),String::from("#ffffff"),String::from("#00ffff"),String::from("#ffff00")
        ,String::from("#ff00ff"),String::from("#ffa500"),String::from("#0000ff"),String::from("#ff0000"),String::from("#00ff00"), String::from("70"), String::from("#000000"), String::from("#000000"), String::from("70")];
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
                                "touch_horiz_sens" => game_settings.touch_horiz_sens=value.parse::<i32>().unwrap_or(25),
                                "down_hold_time" => game_settings.down_hold_time=value.parse::<u32>().unwrap_or(game_settings.down_hold_time),
                                "down_hold_move_interval" => game_settings.down_hold_move_interval=value.parse::<u32>().unwrap_or(game_settings.down_hold_move_interval),
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
        Self{game_settings: Settings::default(), displaying_window: Windows::Game, colors, highscores: Self::get_highscores().unwrap_or_default(), cookies: get_cookies, cookie_notif: false}
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg{
            SettingsMsg::ChangeWindow(w) => {
                self.cookie_notif=false;
                if w==Windows::Highscores{
                    self.highscores=Self::get_highscores().unwrap_or_default();
                }
                self.displaying_window = w;
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
                    7 => {
                        self.game_settings.touch_horiz_sens=value.parse::<i32>().unwrap_or(25)
                    }
                    8 => {
                        self.game_settings.down_hold_time=value.parse::<u32>().unwrap_or(self.game_settings.down_hold_time);
                    }
                    9 => {
                        self.game_settings.down_hold_move_interval=value.parse::<u32>().unwrap_or(self.game_settings.down_hold_move_interval);
                    }
                    _ => {

                    }
                }
            }
            SettingsMsg::SetTheme(theme_id) => {
                match theme_id {
                    1 => {
                        self.colors = vec![String::from("#2c2a29"),String::from("#353231"),String::from("#222120"),String::from("#cfb24a"),String::from("#ffffff"),String::from("#00ffff"),String::from("#ffff00")
                        ,String::from("#ff00ff"),String::from("#ffa500"),String::from("#0000ff"),String::from("#ff0000"),String::from("#00ff00"), String::from("70"), String::from("#000000"), String::from("#000000"), String::from("70")];
                        self.game_settings = Settings::default();
                    }
                    _ => {
                        self.colors = vec![String::from("#2c2a29"),String::from("#333333"),String::from("#222222"),String::from("#a7a7a7"),String::from("#ffffff"),String::from("#00ffff"),String::from("#ffff00")
                        ,String::from("#ff00ff"),String::from("#ffa500"),String::from("#0000ff"),String::from("#ff0000"),String::from("#00ff00"), String::from("70"), String::from("#000000"), String::from("#000000"), String::from("70")];
                        self.game_settings = Settings::default();
                    }
                }
            }
            SettingsMsg::Revert => {
                let get_cookies = document().unchecked_into::<HtmlDocument>().cookie().unwrap_or(String::from("None"));
                self.colors = vec![String::from("#2c2a29"),String::from("#333333"),String::from("#222222"),String::from("#a7a7a7"),String::from("#ffffff"),String::from("#00ffff"),String::from("#ffff00")
                ,String::from("#ff00ff"),String::from("#ffa500"),String::from("#0000ff"),String::from("#ff0000"),String::from("#00ff00"), String::from("70"), String::from("#000000"), String::from("#000000"), String::from("70")];
                self.game_settings = Settings::default();
                if get_cookies!="None"{
                    // colors = Vec::new();
                    for v in get_cookies.split("; "){
                        match v.split_once('='){
                            Some((name, value)) => {
                                if name.starts_with("saved_color_"){
                                    self.colors[name.split_at(12).1.parse::<usize>().unwrap_or(0)] = String::from(value);
                                }else{
                                    match name{
                                        "hold_time" => self.game_settings.hold_time=value.parse::<u32>().unwrap_or(self.game_settings.hold_time),
                                        "hold_move_interval" => self.game_settings.hold_move_interval=value.parse::<u32>().unwrap_or(self.game_settings.hold_move_interval),
                                        "max_switches" => self.game_settings.max_num_held_piece_switches=value.parse::<u32>().unwrap_or(1),
                                        "randomizer" => self.game_settings.randomizer=match value {"Random" => Randomizers::Random, _ => Randomizers::RandomGenerator},
                                        "lock_delay" => self.game_settings.lock_delay=value.parse::<u32>().unwrap_or(500),
                                        "moves_before_lock" => self.game_settings.moves_before_lock=value.parse::<u32>().unwrap_or(15),
                                        "touch_horiz_sens" => self.game_settings.touch_horiz_sens=value.parse::<i32>().unwrap_or(50),
                                        "down_hold_time" => self.game_settings.down_hold_time=value.parse::<u32>().unwrap_or(self.game_settings.down_hold_time),
                                        "down_hold_move_interval" => self.game_settings.down_hold_move_interval=value.parse::<u32>().unwrap_or(self.game_settings.down_hold_move_interval),
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
            }
            SettingsMsg::SaveCookies(force) => {
                let doc = document().unchecked_into::<HtmlDocument>();
                let curr_cookies = doc.cookie().unwrap_or(String::from("None"));
                if curr_cookies.len()<10 && !force{
                    self.cookie_notif=true;
                    return true
                }else if force{
                    self.cookie_notif=false;
                }
                for i in 0..self.colors.len(){
                    let _ = doc.set_cookie(&format!("saved_color_{}={}; expires=Tue, 19 Jan 2038 03:14:07 UTC;",i,self.colors[i]));
                }
                let _ = doc.set_cookie(&format!("hold_time={}; expires=Tue, 19 Jan 2038 03:14:07 UTC;",self.game_settings.hold_time));
                let _ = doc.set_cookie(&format!("hold_move_interval={}; expires=Tue, 19 Jan 2038 03:14:07 UTC;",self.game_settings.hold_move_interval));
                let _ = doc.set_cookie(&format!("max_switches={}; expires=Tue, 19 Jan 2038 03:14:07 UTC;",self.game_settings.max_num_held_piece_switches));
                let _ = doc.set_cookie(&format!("lock_delay={}; expires=Tue, 19 Jan 2038 03:14:07 UTC;",self.game_settings.lock_delay));
                let _ = doc.set_cookie(&format!("moves_before_lock={}; expires=Tue, 19 Jan 2038 03:14:07 UTC;",self.game_settings.moves_before_lock));
                let _ = doc.set_cookie(&format!("randomizer={}; expires=Tue, 19 Jan 2038 03:14:07 UTC;",self.game_settings.randomizer.to_string()));
                let _ = doc.set_cookie(&format!("touch_horiz_sens={}; expires=Tue, 19 Jan 2038 03:14:07 UTC;",self.game_settings.touch_horiz_sens));
                let _ = doc.set_cookie(&format!("down_hold_time={}; expires=Tue, 19 Jan 2038 03:14:07 UTC;",self.game_settings.down_hold_time));
                let _ = doc.set_cookie(&format!("down_hold_move_interval={}; expires=Tue, 19 Jan 2038 03:14:07 UTC;",self.game_settings.down_hold_move_interval));
            }
        }
        true
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let do_no_touch: String = if self.displaying_window==Windows::Game{
            String::from("touch-action:none;")
        }else{
            String::new()
        };
        let doc = document().unchecked_into::<HtmlDocument>();
        let r = doc.query_selector("html, body").unwrap().unwrap();
        let _ = r.set_attribute("style", &format!("background-color: {}; {}",self.colors[0],do_no_touch));
        let link = ctx.link();
        html!{
            <div class="root" style={format!("--bg-color: {}; --board-bg: {}; --board-outline: {}; --text-color: {}; --accent-target: {}; --Icolor: {}; --Ocolor: {}; --Tcolor:{}; --Lcolor: {}; --Jcolor: {};
            --Scolor: {}; --Zcolor: {}; --outline-opacity: {}%; --piece-outline-target: {}; --drop-outline-target: {}; --drop-outline-opacity: {}%;",self.colors[0],self.colors[1], self.colors[2],self.colors[3],self.colors[4],self.colors[5],self.colors[6],
            self.colors[7],self.colors[8],self.colors[9],self.colors[10],self.colors[11],self.colors[12],self.colors[13],self.colors[14],self.colors[15])}>
                <h1>{"Testris"}</h1>
                // <p>{self.colors[0].clone()}</p>
                <div class="windows-buttons">
                    <button class="window-button" onclick={link.callback(|_| SettingsMsg::ChangeWindow(Windows::Game))}>
                    {"🕹️"}
                    </button>
                    <button class="window-button" onclick={link.callback(|_| SettingsMsg::ChangeWindow(Windows::Highscores))}>
                    {"🏆"}
                    </button>
                    <button class="window-button" onclick={link.callback(|_| SettingsMsg::ChangeWindow(Windows::Settings))}>
                    {"⚙️"}
                    </button>
                </div>
                <hr/>
                if self.displaying_window==Windows::Settings{
                    <div class="settings-window">
                    <div class="colors-section">
                    <h1>{"website colors"}</h1>
                    <div class="colors-holder">
                    <div class="color-tab">
                        <input type="color" value={self.colors[0].clone()} onchange={Self::get_color_callback(link,0)}/>
                        <div class="color-tab-text">
                        {"background color"}
                        </div>
                    </div>
                    <div class="color-tab">
                        <input type="color" value={self.colors[1].clone()} onchange={Self::get_color_callback(link,1)}/>
                        <div class="color-tab-text">
                        {"board color"}
                        </div>
                    </div>
                    <div class="color-tab">
                        <input type="color" value={self.colors[2].clone()} onchange={Self::get_color_callback(link,2)}/>
                        <div class="color-tab-text">
                        {"board outline"}
                        </div>
                    </div>
                    <div class="color-tab">
                        <input type="color" value={self.colors[3].clone()} onchange={Self::get_color_callback(link,3)}/>
                        <div class="color-tab-text" style="color: var(--background-color);">
                        {"text color"}
                        </div>
                    </div>
                    <div class="color-tab">
                        <input type="color" value={self.colors[4].clone()} onchange={Self::get_color_callback(link,4)}/>
                        <div class="color-tab-text">
                        {"accent target"}
                        </div>
                    </div>
                    </div>
                    </div>
                    <div class="colors-section">
                        <h1>{"tetrominoes colors"}</h1>
                        <div class="colors-holder">
                            <div class="color-tab">
                                <input type="color" value={self.colors[5].clone()} onchange={Self::get_color_callback(link,5)}/>
                                <div class="color-tab-text" style="color: color-mix(in srgb, var(--Icolor) 20%, var(--board-bg));">
                                {"I tetromino"}
                                </div>
                            </div>
                            <div class="color-tab">
                                <input type="color" value={self.colors[6].clone()} onchange={Self::get_color_callback(link,6)}/>
                                <div class="color-tab-text" style="color: color-mix(in srgb, var(--Ocolor) 20%, var(--board-bg));">
                                {"O tetromino"}
                                </div>
                            </div>
                            <div class="color-tab">
                                <input type="color" value={self.colors[7].clone()} onchange={Self::get_color_callback(link,7)}/>
                                <div class="color-tab-text" style="color: color-mix(in srgb, var(--Tcolor) 20%, var(--board-bg));">
                                {"T tetromino"}
                                </div>
                            </div>
                            <div class="color-tab">
                                <input type="color" value={self.colors[8].clone()} onchange={Self::get_color_callback(link,8)}/>
                                <div class="color-tab-text" style="color: color-mix(in srgb, var(--Lcolor) 20%, var(--board-bg));">
                                {"L tetromino"}
                                </div>
                            </div>    
                            <div class="color-tab">    
                                <input type="color" value={self.colors[9].clone()} onchange={Self::get_color_callback(link,9)}/>
                                <div class="color-tab-text" style="color: color-mix(in srgb, var(--Jcolor) 20%, var(--board-bg));">
                                {"J tetromino"}
                                </div>
                            </div>    
                            <div class="color-tab">
                                <input type="color" value={self.colors[10].clone()} onchange={Self::get_color_callback(link,10)}/>
                                <div class="color-tab-text" style="color: color-mix(in srgb, var(--Scolor) 20%, var(--board-bg));">
                                {"S tetromino"}
                                </div>
                            </div>    
                            <div class="color-tab">
                                <input type="color" value={self.colors[11].clone()} onchange={Self::get_color_callback(link,11)}/>
                                <div class="color-tab-text" style="color: color-mix(in srgb, var(--Zcolor) 20%, var(--board-bg));">
                                {"Z tetromino"}
                                </div>
                            </div>
                            <div class="color-tab">
                                <input type="color" value={self.colors[13].clone()} onchange={Self::get_color_callback(link,13)}/>
                                <div class="color-tab-text" style="color: var(--text-color);">
                                {"piece outline target"}
                                </div>
                            </div>
                            <div class="color-tab">
                                <input type="color" value={self.colors[14].clone()} onchange={Self::get_color_callback(link,14)}/>
                                <div class="color-tab-text" style="color: var(--text-color);">
                                {"drop outline target"}
                                </div>
                            </div>
                        </div>
                    </div>
                    <div class="horiz-section">
                    <h1>{"tetromino outline opacity"}</h1>
                    <div class="text">{"opacity of the outline of tetrominoes relative to the 'piece outline target' (0 = outlines look exactally like the outline target, 100 = outline is same color as piece)"}</div>
                    <input name="piece-outline-opacity" type="range" min=0 max=100 value={self.colors[12].clone()} onchange={Self::get_color_callback(link,12)}/>
                    <h2>{format!("{}%",self.colors[12].clone())}</h2>
                    </div>
                    <div class="horiz-section">
                    <h1>{"drop outline opacity"}</h1>
                    <div class="text">{"opacity of the outline of drop location indicator relative to the 'drop outline target' setting (0 = drop indicator outlines are the same color as 'drop outline target', 100 = drop indicator outlines are the same color as the tetrominoes)"}</div>
                    <input name="piece-outline-opacity" type="range" min=0 max=100 value={self.colors[15].clone()} onchange={Self::get_color_callback(link,15)}/>
                    <h2>{format!("{}%",self.colors[15].clone())}</h2>
                    </div>
                    <div class="horiz-section">
                    <h1>{"key held delay"}</h1>
                    <div class="text">{"The amount of time in milliseconds that a left/right must be held before the piece automatically starts moving sideways while the button remains held."}</div>
                    <input name="key-held-delay" type="number" value={self.game_settings.hold_time.to_string()} onchange={Self::get_settings_callback(link,0)}/>
                    </div>
                    <div class="horiz-section">
                    <h1>{"key held speed"}</h1>
                    <div class="text">{"The time in milliseconds between sideways piece movements when left/right is being held. (changes how fast pieces move side to side when keys are held)"}</div>
                    <input name="key-held-speed" type="number" value={self.game_settings.hold_move_interval.to_string()} onchange={Self::get_settings_callback(link,1)}/>
                    </div>
                    <div class="horiz-section">
                    <h1>{"down key held delay"}</h1>
                    <div class="text">{"The amount of time in milliseconds that down must be held before the piece automatically starts moving down while the button remains held."}</div>
                    <input name="key-held-delay" type="number" value={self.game_settings.down_hold_time.to_string()} onchange={Self::get_settings_callback(link,8)}/>
                    </div>
                    <div class="horiz-section">
                    <h1>{"down key held speed"}</h1>
                    <div class="text">{"The time in milliseconds between downwards piece movement when down is being held. (changes how fast pieces move side to side when keys are held)"}</div>
                    <input name="key-held-speed" type="number" value={self.game_settings.down_hold_move_interval.to_string()} onchange={Self::get_settings_callback(link,9)}/>
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
                    <div class="horiz-section">
                        <h1>{"touch horizontal sensitivity"}</h1>
                        <div class="text">{"Touchscreen sensitivity for moving tetrominoes horizontally side to side"}</div>
                        <input name="moves-before-lock" type="number" value={self.game_settings.touch_horiz_sens.to_string()} onchange={Self::get_settings_callback(link,7)}/>
                    </div>
                    <div class="settings-footer">
                        <button onclick={link.callback(|_| SettingsMsg::Revert)}>{"revert"}</button>
                        <button onclick={link.callback(|_| SettingsMsg::SaveCookies(false))}>{"save"}</button>
                    </div>
                    <p>{self.cookies.clone()}</p>
                    <p>{Self::get_highscores().unwrap().len()}</p>    
                    <p>{Self::get_highscores().unwrap().iter().map(|v| v.to_string()).collect::<Vec<String>>().join(",")}</p>
                    </div>
                    if self.cookie_notif{
                        <div class="cookie-menu">
                            <p>{"Cookies are used to save your preferences. Without cookies, you can still modify your preferences, but they will not be saved once you close this tab."}</p>
                            <button onclick={link.callback(|_| SettingsMsg::ChangeWindow(Windows::Settings))}>{"Reject"}</button>
                            <button onclick={link.callback(|_| SettingsMsg::SaveCookies(true))}>{"Accept"}</button>
                        </div>
                    }
                }else if self.displaying_window==Windows::Highscores{
                    {
                        self.highscores.iter().enumerate().map(|(i,h)| {
                            html!{
                                <div class="highscore-list-item">
                                    {h.to_string()}
                                </div>
                            }
                        }).collect::<Html>()
                    }
                }else{
                <GameDisplay settings={self.game_settings.clone()}/>
                }
            </div>
        }
    }
}
impl RootComponent{
    fn get_color_callback(link: &yew::html::Scope<Self>, val: usize) -> yew::Callback<Event>{
        return link.callback(move |e: Event| {let input: HtmlInputElement = e.target_unchecked_into(); SettingsMsg::ChangeColor(input.value().parse::<String>().unwrap(),val)})
    }
    fn get_settings_callback(link: &yew::html::Scope<Self>, val: u32) -> yew::Callback<Event>{
        return link.callback(move |e: Event| {let input: HtmlInputElement = e.target_unchecked_into(); SettingsMsg::ChangeSettings(input.value().parse::<String>().unwrap(),val)})
    }
    fn get_highscores() -> core::result::Result<Vec<u32>,String>{
        let doc = document().unchecked_into::<HtmlDocument>();
        let curr_cookies = doc.cookie().unwrap_or(String::from("None"));
        if curr_cookies.len()>8{
            if curr_cookies.contains("highscore"){
                let after_hs = String::from(curr_cookies.split_once("highscore=").unwrap_or(("","0")).1);
                if after_hs.contains(";"){
                    return Ok(String::from(curr_cookies.split_once("highscore=").unwrap_or(("","0")).1.split_once(";").unwrap_or(("0","")).0).split(",").map(|v| v.parse::<u32>().unwrap_or(0)).collect());
                }else{
                    return Ok(String::from(curr_cookies.split_once("highscore=").unwrap_or(("","0")).1).split(",").map(|v| v.parse::<u32>().unwrap_or(0)).collect());
                }
            }
            return Err(String::from("Highscore cookie does not exist"))
        }
        return Err(String::from("Cookies not enabled"))
    }
    fn add_highscore(h: u32){
        let doc = document().unchecked_into::<HtmlDocument>();
        match Self::get_highscores(){
            Ok(hs) => {
                if hs.len()<8{
                    let mut cloned=hs.clone();
                    if cloned[0]==0 {cloned.remove(0);}
                    let pos = cloned.binary_search(&h).unwrap_or_else(|e| e);
                    cloned.insert(pos, h);
                    let _ = doc.set_cookie(&format!("highscore={}; expires=Tue, 19 Jan 2038 03:14:07 UTC;",cloned.iter().map(|v| v.to_string()).collect::<Vec<String>>().join(",")));
                }else{
                    let mut cloned: Vec<u32> = hs.clone();
                    if cloned[0]==0 {cloned.remove(0);}
                    let min: u32 = cloned.pop().unwrap_or_default();
                    if min>h{
                        cloned.push(min);
                    }else{
                        let pos = cloned.binary_search(&h).unwrap_or_else(|e| e);
                        cloned.insert(pos, h);
                    }
                    let _ = doc.set_cookie(&format!("highscore={}; expires=Tue, 19 Jan 2038 03:14:07 UTC;",cloned.iter().map(|v| v.to_string()).collect::<Vec<String>>().join(",")));
                };
            },
            Err(s) => if s=="Highscore cookie does not exist" {let _ = doc.set_cookie(&format!("highscore={}; expires=Tue, 19 Jan 2038 03:14:07 UTC;",h));},
        }
        
    }
}

#[derive(Properties,PartialEq, Clone)]
struct GameProps{
    settings: Settings
}

enum GameMsg {
    Left(InputTypes),
    Right(InputTypes),
    Down(InputTypes),
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
    down_handle: (bool,Option<Timeout>),
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
    game_end_screen: bool,
    settings: Settings
}

impl Component for GameDisplay {
    type Message = GameMsg;
    type Properties = GameProps;

    fn create(ctx: &Context<Self>) -> Self {
        let mut piece_queue: VecDeque<TetrisPieceType> = VecDeque::from_iter(ctx.props().settings.randomizer.make_sequence(7).into_iter());
        let first_piece = piece_queue.pop_front().unwrap_or(TetrisPieceType::I);
        GameDisplay { game: TetrisBoard::make(10,20,first_piece), ticker_handle: None, move_handle: (true,None), 
            down_handle: (true,None), settings: ctx.props().settings.clone(), level: 1, stick_handle: None, stick_counter: 0, held_piece: None, held_piece_switch_count: 0,
            piece_queue, score: 0, lines_cleared: 0, game_end_screen: false,
            touch_start_pos: (0,0), touch_pos: (0,0), touch_translation: 0, touch_velocity: (0,0), touch_can_rotate: true}
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            GameMsg::Left(t) => {
                if self.ticker_handle.is_none(){
                    _ctx.link().send_message(GameMsg::Tick);
                }
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
                if self.ticker_handle.is_none(){
                    _ctx.link().send_message(GameMsg::Tick);
                }
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
            GameMsg::Down(t) => {
                // if self.game.move_down(){
                //     self.score+=1;
                // }
                if self.ticker_handle.is_none(){
                    _ctx.link().send_message(GameMsg::Tick);
                }
                if self.down_handle.1.is_none() || t==InputTypes::Hold || self.down_handle.0 && self.down_handle.1.is_some(){
                    if self.game.move_down(){
                        self.score+=1;
                    }
                    if t!=InputTypes::Touch{
                        let handle = if t==InputTypes::Hold{
                            let link = _ctx.link().clone();
                            Timeout::new(self.settings.hold_move_interval, move || link.send_message(GameMsg::Down(InputTypes::Hold)))
                        } else {
                            let link = _ctx.link().clone();
                            Timeout::new(self.settings.hold_time, move || link.send_message(GameMsg::Down(InputTypes::Hold)))
                        };
                        self.down_handle = (false,Some(handle));
                    }
                    if self.stick_counter<self.settings.moves_before_lock{
                        self.stick_handle=None;
                    }
                }
            }
            GameMsg::Drop => {
                if self.ticker_handle.is_none(){
                    _ctx.link().send_message(GameMsg::Tick);
                }
                if self.game_end_screen {return false}
                self.score += self.game.drop()*2;
                if !self.game.new_falling_piece(self.piece_queue.pop_front().unwrap_or(TetrisPieceType::I)){
                    // reset game
                    self.game_end_screen = true;
                    self.ticker_handle=None;
                    RootComponent::add_highscore(self.score);
                    return true
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
                if self.game_end_screen{
                    self.game_end_screen = false;
                    self.game = TetrisBoard::make(10,20,TetrisPieceType::get_random());
                    self.level = 1;
                    self.score = 0;
                    self.lines_cleared=0;
                    self.held_piece=None;
                }
                if !self.game.move_down(){
                    if self.stick_handle.is_none(){
                        self.stick_counter+=1;
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
                if self.ticker_handle.is_none(){
                    _ctx.link().send_message(GameMsg::Tick);
                }
                if self.game_end_screen {return true}
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
                if self.ticker_handle.is_none(){
                    _ctx.link().send_message(GameMsg::Tick);
                }
                self.game.rotate_clockwise();
                if self.stick_counter<self.settings.moves_before_lock{
                    self.stick_handle=None;
                }
            }
            GameMsg::CancelDown => {
                if !self.down_handle.0{self.down_handle=(true,None);}
            }
            GameMsg::CancelLeft => {
                if !self.move_handle.0{self.move_handle=(true,None);}
            }
            GameMsg::CancelRight => {
                if self.move_handle.0{self.move_handle=(true,None);}
            }
            GameMsg::TouchStart(t) => {
                if self.ticker_handle.is_none(){
                    _ctx.link().send_message(GameMsg::Tick);
                }
                let first_touch = t.touches().get(0).unwrap();
                self.touch_start_pos=(first_touch.client_x(),first_touch.client_y());
                self.touch_pos=self.touch_start_pos;
                self.touch_translation=self.touch_start_pos.0;
            }
            GameMsg::TouchMove(t) => {
                let first_touch = t.touches().get(0).unwrap();
                let pos=(first_touch.client_x(),first_touch.client_y());
                if pos.0-self.touch_translation>=self.settings.touch_horiz_sens { // && (pos.1-self.touch_start_pos.1).abs()<self.settings.touch_horiz_sens{
                    self.touch_translation=pos.0;
                    self.touch_can_rotate=false;
                    _ctx.link().send_message(GameMsg::Right(InputTypes::Touch));
                }else if pos.0-self.touch_translation<=-self.settings.touch_horiz_sens { //&& (pos.1-self.touch_start_pos.1).abs()<self.settings.touch_horiz_sens{
                    self.touch_translation=pos.0;
                    self.touch_can_rotate=false;
                    _ctx.link().send_message(GameMsg::Left(InputTypes::Touch));
                }
                if pos.1-self.touch_start_pos.1>80{
                    self.touch_can_rotate=false;
                    _ctx.link().send_message(GameMsg::Down(InputTypes::Touch));
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
            <div class="game no-touch-move" tabindex=0 onkeydown={link.callback(|key:KeyboardEvent| {match key.key_code(){67=>GameMsg::Hold,40=>GameMsg::Down(InputTypes::Tap), 39=>GameMsg::Right(InputTypes::Tap), 38=>GameMsg::Rotate, 37=>GameMsg::Left(InputTypes::Tap), 32 =>GameMsg::Drop,_=>GameMsg::None}})}
            onkeyup={link.callback(|key:KeyboardEvent| {match key.key_code(){40=>GameMsg::CancelDown, 39=>GameMsg::CancelRight, 37=>GameMsg::CancelLeft, 27=>GameMsg::Unfocus, _=>GameMsg::None}})}
            onfocusout={link.callback(|_| GameMsg::Unfocus)}> //onfocusin={link.callback(|_| GameMsg::Tick)} 
                <div class="inline-block" onclick={link.callback(|_| GameMsg::Hold)}>
                    {TetrisPieceType::view(&self.held_piece)}
                    <div class="sidebar-num-display">
                    <h1>{"Score"}</h1>
                    <p>{self.score.to_string()}</p>
                    </div>
                    <div class="sidebar-num-display">
                    <h1>{"Level"}</h1>
                    <p>{self.level.to_string()}</p>
                    </div>
                </div>
                <div class="inline-block" ontouchstart={link.callback(|t:TouchEvent| GameMsg::TouchStart(t))} ontouchmove={link.callback(|t| GameMsg::TouchMove(t))} ontouchend={link.callback(|t| GameMsg::TouchEnd(t))}>
                    {self.game.view()}
                </div>
                <div class="inline-block">
                {
                    (0..self.settings.queue_display_len).map(|v|{
                        html!{TetrisPieceType::view(&Some(self.piece_queue[v].clone()))}
                    }).collect::<Html>()
                }
                </div>

                if self.game_end_screen{
                    <div class="notouch" onclick={link.callback(|_| GameMsg::Tick)}></div>
                    <div class="game-end-menu">
                        <h1>{"Game Over"}</h1>
                        <h2>{format!("Score: {}",self.score)}</h2>
                        <h2>{format!("Level: {}",self.level)}</h2>
                    </div>
                }

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
    down_hold_time: u32,
    down_hold_move_interval: u32,
    max_num_held_piece_switches: u32,
    queue_display_len: usize,
    lock_delay: u32,
    moves_before_lock: u32,
    randomizer: Randomizers,
    touch_horiz_sens: i32
}
impl Default for Settings{
    fn default() -> Settings{
        Settings{hold_time: 150, hold_move_interval: 60, max_num_held_piece_switches: 1, queue_display_len: 4, lock_delay: 500, moves_before_lock: 15, randomizer: Randomizers::RandomGenerator,
        touch_horiz_sens: 25, down_hold_time: 50, down_hold_move_interval: 50}
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
                                            <span class={format!("tile outline drop-indicator {}-color",self.falling_piece)}/>
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
