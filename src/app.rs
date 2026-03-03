use crate::session::Session;

pub const PAGE_SIZE: usize = 5;

pub struct App {
    pub sessions: Vec<Session>,
    pub page: usize,
    pub cursor: usize,
    pub total_pages: usize,
}

impl App {
    pub fn new(sessions: Vec<Session>) -> Self {
        let total_pages = sessions.len().div_ceil(PAGE_SIZE).max(1);
        Self {
            sessions,
            page: 0,
            cursor: 0,
            total_pages,
        }
    }

    pub fn page_sessions(&self) -> &[Session] {
        let s = self.page * PAGE_SIZE;
        &self.sessions[s..(s + PAGE_SIZE).min(self.sessions.len())]
    }

    pub fn selected(&self) -> &Session {
        &self.sessions[self.page * PAGE_SIZE + self.cursor]
    }

    pub fn move_up(&mut self) {
        if self.cursor == 0 {
            if self.page > 0 {
                self.page -= 1;
                self.cursor = PAGE_SIZE - 1;
            }
        } else {
            self.cursor -= 1;
        }
    }

    pub fn move_down(&mut self) {
        let page_len = self.page_sessions().len();
        if self.cursor + 1 >= page_len {
            if self.page + 1 < self.total_pages {
                self.page += 1;
                self.cursor = 0;
            }
        } else {
            self.cursor += 1;
        }
    }

    pub fn prev_page(&mut self) {
        if self.page > 0 {
            self.page -= 1;
            self.cursor = 0;
        }
    }

    pub fn next_page(&mut self) {
        if self.page + 1 < self.total_pages {
            self.page += 1;
            self.cursor = 0;
        }
    }
}
