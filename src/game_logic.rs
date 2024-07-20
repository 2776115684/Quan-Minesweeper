use std::fmt::Display;

use gloo_timers::future::TimeoutFuture;
use leptos::*;
use leptos_router::*;
use rand::{seq::SliceRandom, Rng};
use thiserror::Error;

use crate::{
    game_settings::{Difficulty, ParseDifficultyError, ParseSizeError, Size, Username},
    pages::scores::PostScore,
    utils::to_time,
};

// 定义相邻单元格的坐标偏移
const ADJACENTS: [(isize, isize); 8] = [
    (-1, -1),
    (-1, 0),
    (-1, 1),
    (0, -1),
    (0, 1),
    (1, -1),
    (1, 0),
    (1, 1),
];

// 定义游戏参数解析错误类型
#[derive(Error, Debug)]
pub enum GameParamsError {
    InvalidSize(ParseSizeError),
    InvalidDifficulty(ParseDifficultyError),
}

impl Display for GameParamsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GameParamsError::InvalidSize(err) => err.fmt(f),
            GameParamsError::InvalidDifficulty(err) => err.fmt(f),
        }
    }
}

// 定义游戏参数结构体，包括难度和大小
#[derive(Copy, Clone, PartialEq, Params)]
pub struct GameParams {
    pub difficulty: Difficulty,
    pub size: Size,
}

// 定义游戏状态枚举类型
#[derive(Default, Copy, Clone)]
pub enum GameStatus {
    #[default]
    Idle, // 空闲状态
    Started,  // 游戏开始
    GameOver, // 游戏结束
    Victory,  // 胜利
}

// 定义游戏信息结构体
#[derive(Default)]
pub struct GameInfo {
    elapsed_seconds: i64, // 游戏开始后经过的秒数
    cleared: isize,       // 已清除的单元格数量
    clear_total: isize,   // 总共需要清除的单元格数量
    status: GameStatus,   // 游戏状态
}

// 将游戏信息转换为视图
impl GameInfo {
    pub fn to_view(&self) -> impl IntoView {
        let get_username = move || (expect_context::<ReadSignal<Username>>())().name; // 获取用户名
        let time = to_time(self.elapsed_seconds); // 转换时间为字符串

        match self.status {
            GameStatus::Started => {
                view! {
                    {format!("{} cleared out of {}", self.cleared, self.clear_total)}
                    <br />
                    {time}
                    <br />
                    ""
                    <br />
                }
            }
            GameStatus::GameOver => {
                view! {
                    {move || format!("Game over, {} 😭", get_username())}
                    <br />
                    "Time - " {time}
                    <br />
                    ""
                    <br />
                }
            }
            GameStatus::Victory => {
                view! {
                    {move || format!("You won, {}! 🥳", get_username())}
                    <br />
                    "Time - " {time}
                    <br />
                    ""
                    <br />
                }
            }
            GameStatus::Idle => {
                view! {
                    ""
                    <br />
                    ""
                    <br />
                    ""
                    <br />
                }
            }
        }
    }
}

// 定义单元格交互状态
#[derive(Copy, Clone, Default)]
pub enum CellInteraction {
    #[default]
    Untouched, // 未触及
    Cleared, // 已清除
    Flagged, // 已标记
}

// 定义单元格类型
#[derive(Copy, Clone)]
pub enum CellKind {
    Mine,       // 地雷
    Clear(u32), // 非地雷, 显示数字
}

impl Default for CellKind {
    fn default() -> Self {
        Self::Clear(0)
    }
}

// 定义单元格状态结构体
#[derive(Default, Clone)]
struct CellState {
    interaction: CellInteraction,                             // 交互状态
    kind: CellKind,                                           // 类型
    signal: Option<WriteSignal<(CellInteraction, CellKind)>>, // 用于更新单元格状态的信号
}

impl CellState {
    // 判断是否为地雷
    fn is_mine(&self) -> bool {
        matches!(self.kind, CellKind::Mine)
    }

    // 判断是否为非地雷
    fn is_clear(&self) -> bool {
        matches!(self.kind, CellKind::Clear(_))
    }

    // 判断是否未触及
    fn is_untouched(&self) -> bool {
        matches!(self.interaction, CellInteraction::Untouched)
    }

    // 判断是否已标记
    fn is_flagged(&self) -> bool {
        matches!(self.interaction, CellInteraction::Flagged)
    }
}

// 定义游戏状态结构体
pub struct GameState {
    params: GameParams,
    rows: isize,    // 行数
    columns: isize, // 列数
    mines: isize,
    cleared: isize,
    cell_states: Vec<CellState>,
    status: GameStatus,
    info: ReadSignal<GameInfo>,              // 游戏信息信号
    set_info: WriteSignal<GameInfo>,         // 更新游戏信息信号
    new_game_enabled: ReadSignal<bool>,      // 新游戏按钮是否启用信号
    set_new_game_enabled: WriteSignal<bool>, // // 更新新游戏按钮是否启用信号
    timer: Action<(), ()>,                   // 计时器
}

impl GameState {
    // 各难度模式下的地雷概率
    const EASY_PROB: f64 = 0.15;
    const NORMAL_PROB: f64 = 0.25;
    const HARD_PROB: f64 = 0.35;

    // 各大小模式下的行列数
    const SMALL_SIZE: (isize, isize) = (8, 12);
    const MEDIUM_SIZE: (isize, isize) = (10, 15);
    const LARGE_SIZE: (isize, isize) = (12, 18);

    // 初始化游戏状态
    pub fn new(params: GameParams) -> Self {
        let (rows, columns) = match params.size {
            Size::Small => Self::SMALL_SIZE,
            Size::Medium => Self::MEDIUM_SIZE,
            Size::Large => Self::LARGE_SIZE,
        };
        let total = rows * columns;
        let mines = (total as f64
            * match params.difficulty {
                Difficulty::Easy => Self::EASY_PROB,
                Difficulty::Normal => Self::NORMAL_PROB,
                Difficulty::Hard => Self::HARD_PROB,
            }) as isize;

        let (info, set_info) = create_signal(GameInfo::default());
        set_info.update(|info| info.clear_total = total - mines);

        let timer = create_action(move |&()| async move {
            for second in 0..i64::MAX {
                let mut stop = false;

                let disposed = set_info
                    .try_update(|info| {
                        if matches!(info.status, GameStatus::Started) {
                            info.elapsed_seconds = second;
                        } else {
                            stop = true;
                        }
                    })
                    .is_none();

                if stop || disposed {
                    break;
                }

                TimeoutFuture::new(1_000).await;
            }
        });

        let (new_game_enabled, set_new_game_enabled) = create_signal(true);

        Self {
            params,
            rows,
            columns,
            cell_states: vec![Default::default(); total as usize],
            mines,
            cleared: 0,
            status: Default::default(),
            info,
            set_info,
            new_game_enabled,
            set_new_game_enabled,
            timer,
        }
    }

    // 获取网格尺寸
    pub fn dimensions(&self) -> (isize, isize) {
        (self.rows, self.columns)
    }

    // 获取游戏信息信号
    pub fn info_signal(&self) -> ReadSignal<GameInfo> {
        self.info
    }

    // 获取新游戏按钮是否启用信号
    pub fn new_game_enabled_signal(&self) -> ReadSignal<bool> {
        self.new_game_enabled
    }

    // 开始游戏
    fn start(&mut self, row: isize, column: isize) {
        self.timer.dispatch(());

        let mut rng = rand::thread_rng();

        let exclude = Vec::from_iter(std::iter::once((0, 0)).chain(ADJACENTS).filter_map(
            |(row_offset, column_offset)| self.index(row + row_offset, column + column_offset),
        ));

        for _ in 0..self.mines {
            let cell_state = loop {
                let index = rng.gen_range(0..self.rows * self.columns) as usize;

                if exclude.contains(&index) {
                    continue;
                }

                let cell_state = self.cell_states.get_mut(index).expect("within bounds");

                if !cell_state.is_mine() {
                    break cell_state;
                }
            };

            cell_state.kind = CellKind::Mine;
        }

        for row in 0..self.rows {
            for column in 0..self.columns {
                if self
                    .get_cell_state(row, column)
                    .expect("within bounds")
                    .is_clear()
                {
                    let mines = ADJACENTS
                        .iter()
                        .filter(|(row_offset, column_offset)| {
                            self.get_cell_state(row + row_offset, column + column_offset)
                                .map_or(false, |cell_state| cell_state.is_mine())
                        })
                        .count();

                    self.get_cell_state_mut(row, column)
                        .expect("within bounds")
                        .kind = CellKind::Clear(mines as u32);
                }
            }
        }

        self.status = GameStatus::Started;
    }

    // 获取指定位置的索引
    fn index(&self, row: isize, column: isize) -> Option<usize> {
        (row >= 0 && column >= 0 && row < self.rows && column < self.columns)
            .then_some((row * self.columns + column) as usize)
    }

    // 获取指定位置的单元格状态
    fn get_cell_state(&self, row: isize, column: isize) -> Option<&CellState> {
        self.index(row, column)
            .map(|index| &self.cell_states[index])
    }

    // 获取指定位置的可变单元格状态
    fn get_cell_state_mut(&mut self, row: isize, column: isize) -> Option<&mut CellState> {
        self.index(row, column)
            .map(|index| &mut self.cell_states[index])
    }

    // 注册单元格状态更新信号
    pub fn register_cell(
        &mut self,
        row: isize,
        column: isize,
        set_cell_state: WriteSignal<(CellInteraction, CellKind)>,
    ) {
        self.get_cell_state_mut(row, column)
            .expect("row and column within bounds")
            .signal = Some(set_cell_state);
    }

    // 更新得分
    fn update_score(&mut self) {
        match self.status {
            GameStatus::Started if self.cleared == self.rows * self.columns - self.mines => {
                self.status = GameStatus::Victory;

                for cell_state in &mut self.cell_states {
                    if cell_state.is_untouched() {
                        cell_state.signal.expect("signal registered")((
                            CellInteraction::Flagged,
                            CellKind::Mine,
                        ));
                    }
                }

                let post_score = create_server_action::<PostScore>();

                post_score.dispatch(PostScore {
                    username: (expect_context::<ReadSignal<Username>>())().name,
                    time_in_seconds: self.info.with(|info| info.elapsed_seconds),
                    difficulty: self.params.difficulty,
                    size: self.params.size,
                });
            }

            GameStatus::GameOver => {
                (self.set_new_game_enabled)(false);

                let mut mine_signals = self
                    .cell_states
                    .iter()
                    .filter(|cell_state| cell_state.is_untouched() && cell_state.is_mine())
                    .map(|cell_state| cell_state.signal.expect("signal registered"))
                    .collect::<Vec<_>>();
                mine_signals.shuffle(&mut rand::thread_rng());

                spawn_local({
                    let set_new_game_enabled = self.set_new_game_enabled;

                    async move {
                        TimeoutFuture::new(400).await;

                        for set_cell_state in mine_signals {
                            set_cell_state((CellInteraction::Cleared, CellKind::Mine));
                            TimeoutFuture::new(20).await;
                        }

                        set_new_game_enabled(true);
                    }
                });
            }

            _ => {}
        }

        self.set_info.update(|info| {
            info.cleared = self.cleared;
            info.status = self.status;
        });
    }

    // 挖地雷(挖掘指定位置的单元格)
    pub fn dig(&mut self, row: isize, column: isize) {
        match self.status {
            GameStatus::GameOver | GameStatus::Victory => {
                return;
            }
            GameStatus::Idle => {
                self.start(row, column);
            }
            _ => {}
        }

        self.dig_inner(row, column);
        self.update_score();
    }

    // 挖地雷内部逻辑(扫雷算法的核心)
    fn dig_inner(&mut self, row: isize, column: isize) {
        let Some(cell_state) = self.get_cell_state_mut(row, column) else {
            return;
        };

        // 根据单元格的交互状态进行不同的处理
        match cell_state.interaction {
            CellInteraction::Untouched => {
                cell_state.interaction = CellInteraction::Cleared;

                cell_state.signal.expect("signal registered")((
                    cell_state.interaction,
                    cell_state.kind,
                ));

                match cell_state.kind {
                    CellKind::Mine => {
                        self.status = GameStatus::GameOver;
                        return;
                    }
                    CellKind::Clear(0) => {
                        // 清除0的单元格时(当前单元格周围没有雷且被挖到)，递归清除相邻单元格
                        for (row_offset, column_offset) in ADJACENTS {
                            self.dig_inner(row + row_offset, column + column_offset);
                        }
                    }
                    _ => {}
                }
            }

            // 扫雷游戏中的"安全点击策略", 当点击一个已被清除的单元格时, 如果周围的旗帜数量等于该单元格的数字, 则挖开周围未被挖开的单元格
            // 给用户提供一种快捷的扫雷方式
            // 但如果旗子位置不正确, 使用安全点击则会引爆地雷, 导致游戏失败
            CellInteraction::Cleared => {
                if let CellKind::Clear(mines) = self
                    .get_cell_state(row, column)
                    .expect("within bounds")
                    .kind
                {
                    let flags = ADJACENTS
                        .iter()
                        .filter(|(row_offset, column_offset)| {
                            self.get_cell_state(row + row_offset, column + column_offset)
                                .map_or(false, |cell_state| cell_state.is_flagged())
                        })
                        .count();

                    // 比较地雷数量和旗子数量
                    if mines == flags as u32 {
                        for (row_offset, column_offset) in ADJACENTS {
                            if let Some(cell_state) =
                                self.get_cell_state(row + row_offset, column + column_offset)
                            {
                                if cell_state.is_untouched() {
                                    self.dig_inner(row + row_offset, column + column_offset);
                                }
                            }
                        }
                    }
                }

                return;
            }

            // 已标记状态下不允许挖开单元格
            CellInteraction::Flagged => {
                return;
            }
        }

        self.cleared += 1;
    }

    // 标记或取消标记指定位置的单元格(插旗或拔旗)
    pub fn flag(&mut self, row: isize, column: isize) {
        if matches!(self.status, GameStatus::GameOver | GameStatus::Victory) {
            return;
        }

        let Some(cell_state) = self.get_cell_state_mut(row, column) else {
            return;
        };

        match cell_state.interaction {
            CellInteraction::Untouched => {
                cell_state.interaction = CellInteraction::Flagged;
            }
            CellInteraction::Cleared => {
                return;
            }
            CellInteraction::Flagged => {
                cell_state.interaction = CellInteraction::Untouched;
            }
        }

        cell_state.signal.expect("signal registered")((cell_state.interaction, cell_state.kind));
    }

    // 重置游戏状态
    pub fn reset(&mut self) {
        self.status = Default::default();
        self.cleared = Default::default();

        for cell_state in &mut self.cell_states {
            cell_state.interaction = Default::default();
            cell_state.kind = Default::default();

            if let Some(cell_state_signal) = cell_state.signal {
                cell_state_signal((Default::default(), Default::default()));
            }
        }

        (self.set_info)(GameInfo {
            clear_total: self.rows * self.columns - self.mines,
            ..Default::default()
        });
    }
}
