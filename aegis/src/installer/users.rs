use ratatui::{crossterm::event::KeyCode, layout::Constraint, text::Line};
use serde::{Deserialize, Serialize};
use crate::{
  installer::{HIGHLIGHT, Installer, Page, Signal},
  split_hor, split_vert, styled_block, ui_back, ui_close, ui_down, ui_enter, ui_up,
  widget::{
    Button, ConfigWidget, HelpModal, InfoBox, LineEditor, StrList, TableWidget,
    WidgetBox,
  },
};

fn normalize_and_validate_username(raw: &str) -> Result<String, &'static str> {
  let s = raw.trim().to_lowercase();
  if s.is_empty() {
    return Err("Username cannot be empty");
  }
  // Only allow ASCII letters and digits: a–z, 0–9
  if !s.chars().all(|c| c.is_ascii_alphanumeric()) {
    return Err("Special characters are not allowed (letters and digits only)");
  }
  Ok(s)
}

fn default_shell() -> String { "bash".to_string() }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
  pub username: String,
  pub password_hash: String,
  pub groups: Vec<String>,
  #[serde(default = "default_shell")]
  pub shell: String,
}

impl User {
  pub fn as_table_row(&self) -> Vec<String> {
    let groups = if self.groups.is_empty() {
      "<none>".to_string()
    } else {
      self.groups.join(", ")
    };
    vec![self.username.clone(), groups, self.shell.clone()]
  }
}

pub struct UserAccounts {
  pub user_table: TableWidget,
  pub buttons: WidgetBox,
  help_modal: HelpModal<'static>,
}

impl UserAccounts {
  pub fn new(users: Vec<User>) -> Self {
    let buttons = vec![Box::new(Button::new("Back")) as Box<dyn ConfigWidget>];
    let buttons = WidgetBox::button_menu(buttons);
    let widths = vec![
      Constraint::Percentage(33),
      Constraint::Percentage(33),
      Constraint::Percentage(33),
    ];
    let headers = vec![
      "Username".to_string(),
      "Groups".to_string(),
      "Shell".to_string(),
    ];
    let mut rows: Vec<Vec<String>> = users.into_iter().map(|u| u.as_table_row()).collect();
    rows.insert(0, vec!["Add a new user".into(), "".into(), "".into()]);
    let mut user_table = TableWidget::new("Users", widths, headers, rows);
    user_table.focus();
    let help_content = styled_block(vec![
      vec![
        (
          Some((
            ratatui::style::Color::Yellow,
            ratatui::style::Modifier::BOLD,
          )),
          "↑/↓, j/k",
        ),
        (None, " - Navigate user list"),
      ],
      vec![
        (
          Some((
            ratatui::style::Color::Yellow,
            ratatui::style::Modifier::BOLD,
          )),
          "Enter →, l",
        ),
        (None, " - Add new user or edit selected user"),
      ],
      vec![
        (
          Some((
            ratatui::style::Color::Yellow,
            ratatui::style::Modifier::BOLD,
          )),
          "Tab",
        ),
        (None, " - Switch between user list and buttons"),
      ],
      vec![
        (
          Some((
            ratatui::style::Color::Yellow,
            ratatui::style::Modifier::BOLD,
          )),
          "Esc, q, ←, h",
        ),
        (None, " - Return to main menu"),
      ],
      vec![
        (
          Some((
            ratatui::style::Color::Yellow,
            ratatui::style::Modifier::BOLD,
          )),
          "?",
        ),
        (None, " - Show this help"),
      ],
      vec![(None, "")],
      vec![(None, "Create user accounts for your system.")],
      vec![(
        None,
        "Select 'Add a new user' to create accounts, or select",
      )],
      vec![(None, "an existing user to modify their settings.")],
    ]);
    let help_modal = HelpModal::new("User Accounts", help_content);
    Self {
      user_table,
      buttons,
      help_modal,
    }
  }
  pub fn display_widget(installer: &mut Installer) -> Option<Box<dyn ConfigWidget>> {
    let users = installer.users.clone();
    if users.is_empty() {
      return None;
    }
    Some(Box::new(TableWidget::new(
      "Users",
      vec![
        Constraint::Percentage(33),
        Constraint::Percentage(33),
        Constraint::Percentage(33),
      ],
      vec![
        "Username".to_string(),
        "Groups".to_string(),
        "Shell".to_string(),
      ],
      users.into_iter().map(|u| u.as_table_row()).collect(),
    )))
  }
  pub fn page_info<'a>() -> (String, Vec<Line<'a>>) {
    let title = "User Accounts".to_string();
    let description = vec![Line::from("Manage user accounts for the system.")];
    (title, description)
  }
}

impl Page for UserAccounts {
  fn render(
    &mut self,
    installer: &mut super::Installer,
    f: &mut ratatui::Frame,
    area: ratatui::prelude::Rect,
  ) {
    let chunks = split_vert!(
      area,
      1,
      [Constraint::Percentage(60), Constraint::Percentage(40),]
    );
    let mut rows: Vec<Vec<String>> = installer
      .users
      .clone()
      .into_iter()
      .map(|u| u.as_table_row())
      .collect();
    rows.insert(0, vec!["Add a new user".into(), "".into(), "".into()]);
    self.user_table.set_rows(rows);
    self.user_table.fix_selection();
    self.user_table.render(f, chunks[0]);
    self.buttons.render(f, chunks[1]);

    // Render help modal on top
    self.help_modal.render(f, area);
  }

  fn handle_input(
    &mut self,
    installer: &mut super::Installer,
    event: ratatui::crossterm::event::KeyEvent,
  ) -> Signal {
    match event.code {
      KeyCode::Char('?') => {
        self.help_modal.toggle();
        return Signal::Wait;
      }
      ui_close!() if self.help_modal.visible => {
        self.help_modal.hide();
        return Signal::Wait;
      }
      _ if self.help_modal.visible => {
        return Signal::Wait;
      }
      _ => {}
    }

    if self.user_table.is_focused() {
      match event.code {
        ui_down!() => {
          if !self.user_table.next_row() {
            self.user_table.unfocus();
            self.buttons.focus();
            self.buttons.first_child();
          }
          Signal::Wait
        }
        ui_up!() => {
          if !self.user_table.previous_row() {
            self.user_table.unfocus();
            self.buttons.focus();
            self.buttons.last_child();
          }
          Signal::Wait
        }
        ui_enter!() => {
          let Some(selected_user) = self.user_table.selected_row() else {
            return Signal::Error(anyhow::anyhow!("No user selected"));
          };
          if selected_user == 0 {
            // Add a new user
            Signal::Push(Box::new(AddUser::new()))
          } else {
            let groups = installer
              .users
              .get(selected_user - 1)
              .map(|u| u.groups.clone())
              .unwrap_or_default();
            Signal::Push(Box::new(AlterUser::new(selected_user - 1, groups)))
          }
        }
        ui_back!() => Signal::Pop,
        _ => Signal::Wait,
      }
    } else if self.buttons.is_focused() {
      match event.code {
        ui_down!() => {
          if !self.buttons.next_child() {
            self.buttons.unfocus();
            self.user_table.focus();
            self.user_table.first_row();
          }
          Signal::Wait
        }
        ui_up!() => {
          if !self.buttons.prev_child() {
            self.buttons.unfocus();
            self.user_table.focus();
            self.user_table.last_row();
          }
          Signal::Wait
        }
        ui_enter!() => {
          match self.buttons.selected_child() {
            Some(0) => {
              // Back
              Signal::Pop
            }
            _ => Signal::Wait,
          }
        }
        ui_back!() => Signal::Pop,
        _ => Signal::Wait,
      }
    } else {
      self.buttons.focus();
      Signal::Wait
    }
  }

  fn get_help_content(&self) -> (String, Vec<Line<'_>>) {
    let help_content = styled_block(vec![
      vec![
        (
          Some((
            ratatui::style::Color::Yellow,
            ratatui::style::Modifier::BOLD,
          )),
          "↑/↓, j/k",
        ),
        (None, " - Navigate user list"),
      ],
      vec![
        (
          Some((
            ratatui::style::Color::Yellow,
            ratatui::style::Modifier::BOLD,
          )),
          "Enter, →, j",
        ),
        (None, " - Add new user or edit selected user"),
      ],
      vec![
        (
          Some((
            ratatui::style::Color::Yellow,
            ratatui::style::Modifier::BOLD,
          )),
          "Tab",
        ),
        (None, " - Switch between user list and buttons"),
      ],
      vec![
        (
          Some((
            ratatui::style::Color::Yellow,
            ratatui::style::Modifier::BOLD,
          )),
          "Esc, q, ←, h",
        ),
        (None, " - Return to main menu"),
      ],
      vec![
        (
          Some((
            ratatui::style::Color::Yellow,
            ratatui::style::Modifier::BOLD,
          )),
          "?",
        ),
        (None, " - Show this help"),
      ],
      vec![(None, "")],
      vec![(None, "Create user accounts for your system.")],
      vec![(
        None,
        "Select 'Add a new user' to create accounts, or select",
      )],
      vec![(None, "an existing user to modify their settings.")],
    ]);
    ("User Accounts".to_string(), help_content)
  }
}

pub struct AddUser {
  name_input: LineEditor,
  pass_input: LineEditor,
  pass_confirm: LineEditor,
  shell_list: StrList,
  selected_shell: String,
  help_modal: HelpModal<'static>,

  username: Option<String>,
  finished: bool,
  created_user_idx: Option<usize>,
}

impl AddUser {
  pub fn new() -> Self {
    let mut name_input = LineEditor::new("Username", None::<&str>);
    name_input.focus();

    let shell_list = StrList::new("Shell", vec![
      "bash".to_string(),
      "fish".to_string(),
      "zsh".to_string(),
    ]);

    let help_content = styled_block(vec![
      vec![
        (
          Some((
            ratatui::style::Color::Yellow,
            ratatui::style::Modifier::BOLD,
          )),
          "Tab",
        ),
        (None, " - Move to next field"),
      ],
      vec![
        (
          Some((
            ratatui::style::Color::Yellow,
            ratatui::style::Modifier::BOLD,
          )),
          "Shift+Tab",
        ),
        (None, " - Move to previous field"),
      ],
      vec![
        (
          Some((
            ratatui::style::Color::Yellow,
            ratatui::style::Modifier::BOLD,
          )),
          "Enter",
        ),
        (None, " - Create user account"),
      ],
      vec![
        (
          Some((
            ratatui::style::Color::Yellow,
            ratatui::style::Modifier::BOLD,
          )),
          "Esc",
        ),
        (None, " - Cancel and return"),
      ],
      vec![
        (
          Some((
            ratatui::style::Color::Yellow,
            ratatui::style::Modifier::BOLD,
          )),
          "←/→",
        ),
        (None, " - Move cursor in text field"),
      ],
      vec![
        (
          Some((
            ratatui::style::Color::Yellow,
            ratatui::style::Modifier::BOLD,
          )),
          "Home/End",
        ),
        (None, " - Jump to field beginning/end"),
      ],
      vec![
        (
          Some((
            ratatui::style::Color::Yellow,
            ratatui::style::Modifier::BOLD,
          )),
          "?",
        ),
        (None, " - Show this help"),
      ],
      vec![(None, "")],
      vec![(None, "Create a new user account for your system.")],
      vec![(None, "Enter username, password, and confirm password.")],
      vec![(None, "Passwords are hidden during entry for security.")],
    ]);
    let help_modal = HelpModal::new("Add User", help_content);
    Self {
      name_input,
      pass_input: LineEditor::new("Password", None::<&str>).secret(true),
      pass_confirm: LineEditor::new("Confirm Password", None::<&str>).secret(true),
      shell_list,
      selected_shell: "bash".into(),
      help_modal,
      username: None,
      finished: false,
      created_user_idx: None,
    }
  }
  pub fn cycle_forward(&mut self) {
    if self.name_input.is_focused() {
      let entered = self
        .name_input
        .get_value()
        .and_then(|s| s.as_str().map(|s| s.to_owned()))
        .unwrap_or_default();

      match normalize_and_validate_username(&entered) {
        Ok(norm) => {
          self.name_input.clear_error();
          self.username = Some(norm);
          self.name_input.unfocus();
          self.pass_input.focus();
        }
        Err(msg) => {
          self.name_input.error(msg);
        }
      }
    } else if self.pass_input.is_focused() {
        self.pass_input.unfocus();
        self.pass_confirm.focus();
    } else if self.pass_confirm.is_focused() {
        let pass = self
          .pass_input
          .get_value()
          .and_then(|v| v.as_str().map(|s| s.to_owned()))
          .unwrap_or_default();

        let confirm = self
          .pass_confirm
          .get_value()
          .and_then(|v| v.as_str().map(|s| s.to_owned()))
          .unwrap_or_default();

        if pass.is_empty() {
          self.pass_input.error("Password cannot be empty");
          self.pass_confirm.unfocus();
          self.pass_input.focus();
          return;
        }
        if confirm.is_empty() {
          self.pass_confirm.error("Password confirmation cannot be empty");
          return;
        }
        if pass != confirm {
          self.pass_confirm.clear();
          self.pass_confirm.error("Passwords do not match");
          self.pass_input.unfocus();
          self.pass_confirm.focus();
          return;
        }
        self.pass_input.clear_error();
        self.pass_confirm.clear_error();
        self.pass_confirm.unfocus();
        self.shell_list.focus();
    } else if self.shell_list.is_focused() {
        self.shell_list.unfocus();
        self.name_input.focus(); // wrap
    } else {
        self.name_input.focus();
    }
  }
  pub fn cycle_backward(&mut self) {
    if self.name_input.is_focused() {
      self.name_input.unfocus();
      self.shell_list.focus(); // wrap backwards
    } else if self.pass_input.is_focused() {
        self.pass_input.unfocus();
        self.name_input.focus();
    } else if self.pass_confirm.is_focused() {
        self.pass_confirm.unfocus();
        self.pass_input.focus();
    } else if self.shell_list.is_focused() {
        self.shell_list.unfocus();
        self.pass_confirm.focus();
    } else {
        self.name_input.focus();
    }
  }
}

impl Default for AddUser {
  fn default() -> Self {
    Self::new()
  }
}

impl Page for AddUser {
  fn render(
    &mut self,
    _installer: &mut super::Installer,
    f: &mut ratatui::Frame,
    area: ratatui::prelude::Rect,
  ) {
    let hor_chunks = split_hor!(
      area,
      1,
      [
        Constraint::Percentage(25),
        Constraint::Percentage(50),
        Constraint::Percentage(25),
      ]
    );
    let chunks = split_vert!(
      hor_chunks[1],
      0,
      [
        Constraint::Length(5), // username
        Constraint::Length(5), // password
        Constraint::Length(5), // password confirm
        Constraint::Length(5), // shell
        Constraint::Min(0),
      ]
    );
    self.name_input.render(f, chunks[0]);
    self.pass_input.render(f, chunks[1]);
    self.pass_confirm.render(f, chunks[2]);
    self.shell_list.render(f, chunks[3]);

    // Render help modal on top
    self.help_modal.render(f, area);
  }

fn handle_input(
  &mut self,
  installer: &mut super::Installer,
  event: ratatui::crossterm::event::KeyEvent,
) -> Signal {

  if self.finished {
    return Signal::Pop;
  }

  match event.code {
    KeyCode::Char('?') => {
      self.help_modal.toggle();
      return Signal::Wait;
    }
    ui_close!() if self.help_modal.visible => {
      self.help_modal.hide();
      return Signal::Wait;
    }
    KeyCode::Esc => return Signal::Pop,
    _ if self.help_modal.visible => {
      return Signal::Wait;
    }
    _ => {}
  }

  if event.code == KeyCode::Tab {
    self.cycle_forward();
    return Signal::Wait;
  } else if event.code == KeyCode::BackTab {
    self.cycle_backward();
    return Signal::Wait;
  }

  if self.name_input.is_focused() {
    match event.code {
      KeyCode::Enter => {
        let entered_owned = self
          .name_input
          .get_value()
          .and_then(|s| s.as_str().map(|s| s.to_owned()))
          .unwrap_or_default();

        let normalized = match normalize_and_validate_username(&entered_owned) {
          Ok(n) => n,
          Err(msg) => { self.name_input.error(msg); return Signal::Wait; }
        };

        if installer.users.iter().any(|u| u.username == normalized) {
          self.name_input.error("User already exists");
          return Signal::Wait;
        }

        self.name_input.clear_error();
        self.username = Some(normalized);
        self.name_input.unfocus();
        self.pass_input.focus();
        Signal::Wait
      }
      KeyCode::Esc => Signal::Pop,
      _ => self.name_input.handle_input(event),
    }
  } else if self.pass_input.is_focused() {
    match event.code {
      KeyCode::Enter => {
        if let Some(pass) = self.pass_input.get_value() {
          let Some(pass) = pass.as_str() else {
            self.pass_input.error("Password cannot be empty");
            return Signal::Wait;
          };
          if pass.is_empty() {
            self.pass_input.error("Password cannot be empty");
            return Signal::Wait;
          }
          self.pass_input.clear_error();
          self.pass_input.unfocus();
          self.pass_confirm.focus();
          Signal::Wait
        } else {
          self.pass_input.error("Password cannot be empty");
          Signal::Wait
        }
      }
      _ => self.pass_input.handle_input(event),
    }
  } else if self.pass_confirm.is_focused() {
    match event.code {
      KeyCode::Enter => {
        if let Some(pass) = self.pass_input.get_value() {
          let Some(pass) = pass.as_str() else {
            self.pass_input.error("Password cannot be empty");
            return Signal::Wait;
          };
          if let Some(confirm) = self.pass_confirm.get_value() {
            let Some(confirm) = confirm.as_str() else {
              self
                .pass_confirm
                .error("Password confirmation cannot be empty");
              return Signal::Wait;
            };
            if pass != confirm {
              self.pass_confirm.clear();
              self.pass_confirm.error("Passwords do not match");
              self.pass_input.unfocus();
              self.pass_confirm.focus();
              return Signal::Wait;
            }

            // Keep your original flow: hash + validate username + dup check
            // (we'll finalize after picking shell)
            let _hashed = match super::RootPassword::mkpasswd(pass.to_string()) {
              Ok(h) => h,
              Err(e) => {
                return Signal::Error(anyhow::anyhow!("Failed to hash password: {e}"));
              }
            };

            let username = match normalize_and_validate_username(
              &self.username.clone().unwrap_or_default()
            ) {
              Ok(n) => n,
              Err(msg) => {
                self.name_input.error(msg);
                self.pass_confirm.unfocus();
                self.name_input.focus();
                return Signal::Wait;
              }
            };

            if installer.users.iter().any(|u| u.username == username) {
              // bounce focus back to the username field so the user can fix it
              self.name_input.error("User already exists");
              self.pass_confirm.unfocus();
              self.name_input.focus();
              return Signal::Wait;
            }

            // ---- Minimal change: instead of creating the user here,
            // show the shell picker (bash/fish/zsh) and finalize there.
            self.username = Some(username);  // keep normalized

            self.pass_input.clear_error();
            self.pass_confirm.clear_error();

            self.pass_confirm.unfocus();
            self.shell_list.focus();
            Signal::Wait
            // ---- end change
          } else {
            self
              .pass_confirm
              .error("Password confirmation cannot be empty");
            Signal::Wait
          }
        } else {
          self.pass_input.error("Password cannot be empty");
          Signal::Wait
        }
      }
      _ => self.pass_confirm.handle_input(event),
    }
  } else if self.shell_list.is_focused() {
    match event.code {
      ui_down!() => {
        self.shell_list.next_item();
        Signal::Wait
      }
      ui_up!() => {
        self.shell_list.previous_item();
        Signal::Wait
      }
      KeyCode::Tab => {
        self.cycle_forward();
        Signal::Wait
      }
      KeyCode::BackTab => {
        self.cycle_backward();
        Signal::Wait
      }
      KeyCode::Esc => {
        // back to confirm step
        self.shell_list.unfocus();
        self.pass_confirm.focus();
        Signal::Wait
      }
      KeyCode::Enter => {
        // finalize: read shell, re-validate username, re-hash, then push user
        if let Some(sel) = self.shell_list.selected_item() {
          self.selected_shell = sel.to_string();
        }

        let pass = self
          .pass_input
          .get_value()
          .and_then(|v| v.as_str().map(|s| s.to_owned()))
          .unwrap_or_default();

        let confirm = self
          .pass_confirm
          .get_value()
          .and_then(|v| v.as_str().map(|s| s.to_owned()))
          .unwrap_or_default();

        if pass.is_empty() {
          self.pass_input.error("Password cannot be empty");
          self.shell_list.unfocus();
          self.pass_input.focus();
          return Signal::Wait;
        }
        if confirm.is_empty() {
          self.pass_confirm.error("Password confirmation cannot be empty");
          self.shell_list.unfocus();
          self.pass_confirm.focus();
          return Signal::Wait;
        }
        if pass != confirm {
          self.pass_confirm.clear();
          self.pass_confirm.error("Passwords do not match");
          self.pass_input.unfocus();
          self.pass_confirm.focus();
          return Signal::Wait;
        }

        let username = match normalize_and_validate_username(
          &self.username.clone().unwrap_or_default()
        ) {
          Ok(n) => n,
          Err(msg) => {
            self.name_input.error(msg);
            self.shell_list.unfocus();
            self.name_input.focus();
            return Signal::Wait;
          }
        };

        if installer.users.iter().any(|u| u.username == username) {
          self.name_input.error("User already exists");
          self.shell_list.unfocus();
          self.name_input.focus();
          return Signal::Wait;
        }

        self.name_input.clear_error();
        self.pass_input.clear_error();
        self.pass_confirm.clear_error();

        let pass = self
          .pass_input
          .get_value()
          .and_then(|s| s.as_str().map(|s| s.to_owned()))
          .unwrap_or_default();

        let hashed = match super::RootPassword::mkpasswd(pass) {
          Ok(h) => h,
          Err(e) => {
            return Signal::Error(anyhow::anyhow!("Failed to hash password: {e}"));
          }
        };

        installer.users.push(User {
          username,
          password_hash: hashed,
          groups: vec![],
          shell: self.selected_shell.clone(),
        });
        let idx = installer.users.len() - 1;
        self.created_user_idx = Some(idx);
        self.finished = true;

        let groups_page = AlterUser::focus_edit_groups(idx, installer.users[idx].groups.clone());

        Signal::Push(Box::new(groups_page))
      }
      _ => Signal::Wait,
    }
  } else {
    self.name_input.focus();
    Signal::Wait
  }
}

  fn get_help_content(&self) -> (String, Vec<Line<'_>>) {
    let help_content = styled_block(vec![
      vec![
        (
          Some((
            ratatui::style::Color::Yellow,
            ratatui::style::Modifier::BOLD,
          )),
          "Tab",
        ),
        (None, " - Move to next field"),
      ],
      vec![
        (
          Some((
            ratatui::style::Color::Yellow,
            ratatui::style::Modifier::BOLD,
          )),
          "Shift+Tab",
        ),
        (None, " - Move to previous field"),
      ],
      vec![
        (
          Some((
            ratatui::style::Color::Yellow,
            ratatui::style::Modifier::BOLD,
          )),
          "Enter",
        ),
        (None, " - Create user account"),
      ],
      vec![
        (
          Some((
            ratatui::style::Color::Yellow,
            ratatui::style::Modifier::BOLD,
          )),
          "Esc",
        ),
        (None, " - Cancel and return"),
      ],
      vec![
        (
          Some((
            ratatui::style::Color::Yellow,
            ratatui::style::Modifier::BOLD,
          )),
          "←/→",
        ),
        (None, " - Move cursor in text field"),
      ],
      vec![
        (
          Some((
            ratatui::style::Color::Yellow,
            ratatui::style::Modifier::BOLD,
          )),
          "Home/End",
        ),
        (None, " - Jump to field beginning/end"),
      ],
      vec![
        (
          Some((
            ratatui::style::Color::Yellow,
            ratatui::style::Modifier::BOLD,
          )),
          "?",
        ),
        (None, " - Show this help"),
      ],
      vec![(None, "")],
      vec![(None, "Create a new user account for your system.")],
      vec![(None, "Enter username, password, and confirm password.")],
      vec![(None, "Passwords are hidden during entry for security.")],
    ]);
    ("Add User".to_string(), help_content)
  }
}

pub struct AlterUser {
  pub selected_user: usize,

  /// Main menu options
  pub buttons: WidgetBox,

  /// Name change
  pub name_input: LineEditor,

  /// Password change
  pub pass_input: LineEditor,
  pub pass_confirm: LineEditor,
  pub shell_list: StrList,

  /// Group Editor
  pub group_name_input: LineEditor,
  pub group_list: StrList,
  help_modal: HelpModal<'static>,
  confirming_delete: bool,
  groups_only: bool,
}

impl AlterUser {
  pub fn new(selected_user_idx: usize, groups: Vec<String>) -> Self {
    let buttons = vec![
      Box::new(Button::new("Change username")) as Box<dyn ConfigWidget>,
      Box::new(Button::new("Change password")) as Box<dyn ConfigWidget>,
      Box::new(Button::new("Change shell")) as Box<dyn ConfigWidget>,
      Box::new(Button::new("Edit Groups")) as Box<dyn ConfigWidget>,
      Box::new(Button::new("Delete user")) as Box<dyn ConfigWidget>,
    ];
    let mut buttons = WidgetBox::button_menu(buttons);
    let shell_list = StrList::new("Shell", vec![
      "bash".to_string(),
      "fish".to_string(),
      "zsh".to_string(),
    ]);
    buttons.focus();
    let help_content = styled_block(vec![
      vec![
        (
          Some((
            ratatui::style::Color::Yellow,
            ratatui::style::Modifier::BOLD,
          )),
          "↑/↓, j/k",
        ),
        (None, " - Navigate menu options"),
      ],
      vec![
        (
          Some((
            ratatui::style::Color::Yellow,
            ratatui::style::Modifier::BOLD,
          )),
          "Enter, →, l",
        ),
        (None, " - Select option"),
      ],
      vec![
        (
          Some((
            ratatui::style::Color::Yellow,
            ratatui::style::Modifier::BOLD,
          )),
          "Tab",
        ),
        (None, " - Navigate between fields"),
      ],
      vec![
        (
          Some((
            ratatui::style::Color::Yellow,
            ratatui::style::Modifier::BOLD,
          )),
          "Esc, q, ←, h",
        ),
        (None, " - Return to previous menu"),
      ],
      vec![
        (
          Some((
            ratatui::style::Color::Yellow,
            ratatui::style::Modifier::BOLD,
          )),
          "?",
        ),
        (None, " - Show this help"),
      ],
      vec![(None, "")],
      vec![(
        None,
        "Modify an existing user account. Choose from changing",
      )],
      vec![(None, "username, password, groups, or deleting the user.")],
    ]);
    let help_modal = HelpModal::new("Alter User", help_content);
    Self {
      selected_user: selected_user_idx,
      buttons,
      name_input: LineEditor::new("New username", None::<&str>),
      pass_input: LineEditor::new("New password", None::<&str>).secret(true),
      pass_confirm: LineEditor::new("Confirm password", None::<&str>).secret(true),
      shell_list,
      group_name_input: LineEditor::new("Add group", None::<&str>),
      group_list: StrList::new("Groups", groups),
      help_modal,
      confirming_delete: false,
      groups_only: false,
    }
  }
  pub fn focus_edit_groups(selected_user_idx: usize, groups: Vec<String>) -> Self {
    let mut s = Self::new(selected_user_idx, groups);
    s.groups_only = true;
    s.buttons.unfocus();
    s.group_name_input.focus(); // this makes the editor render() the Edit Groups view
    s
  }
  pub fn render_main_menu(&mut self, f: &mut ratatui::Frame, area: ratatui::prelude::Rect) {
    let vert_chunks = split_vert!(
      area,
      1,
      [Constraint::Percentage(50), Constraint::Percentage(50)]
    );
    let hor_chunks = split_hor!(
      vert_chunks[0],
      1,
      [
        Constraint::Percentage(40),
        Constraint::Percentage(20),
        Constraint::Percentage(40),
      ]
    );
    self.buttons.render(f, hor_chunks[1]);
  }
  pub fn render_name_change(&mut self, f: &mut ratatui::Frame, area: ratatui::prelude::Rect) {
    // Give room for the error/help line under the input
    let chunks = split_vert!(area, 1, [Constraint::Length(7), Constraint::Min(0)]);
    let hor_chunks = split_hor!(
      chunks[0],
      1,
      [
        Constraint::Percentage(25),
        Constraint::Percentage(50),
        Constraint::Percentage(25),
      ]
    );
    self.name_input.render(f, hor_chunks[1]);
  }
  pub fn render_pass_change(&mut self, f: &mut ratatui::Frame, area: ratatui::prelude::Rect) {
    let chunks = split_vert!(
      area,
      1,
      [
        Constraint::Length(7),
        Constraint::Length(7),
        Constraint::Min(0),
      ]
    );
    let hor_chunks1 = split_hor!(
      chunks[0],
      1,
      [
        Constraint::Percentage(25),
        Constraint::Percentage(50),
        Constraint::Percentage(25),
      ]
    );
    let hor_chunks2 = split_hor!(
      chunks[1],
      1,
      [
        Constraint::Percentage(25),
        Constraint::Percentage(50),
        Constraint::Percentage(25),
      ]
    );
    self.pass_input.render(f, hor_chunks1[1]);
    self.pass_confirm.render(f, hor_chunks2[1]);
  }
  pub fn render_select_shell(&mut self, f: &mut ratatui::Frame, area: ratatui::prelude::Rect) {
    let hor_chunks = split_hor!(area, 1, [
      Constraint::Percentage(25),
      Constraint::Percentage(50),
      Constraint::Percentage(25),
    ]);
    let vert = split_vert!(hor_chunks[1], 1, [Constraint::Length(7), Constraint::Min(0)]);
    self.shell_list.render(f, vert[0]);
  }
  pub fn render_edit_groups(
    &mut self,
    _installer: &mut super::Installer,
    f: &mut ratatui::Frame,
    area: ratatui::prelude::Rect,
  ) {
    let hor_chunks = split_hor!(
      area,
      1,
      [Constraint::Percentage(50), Constraint::Percentage(50)]
    );
    let line_editor_chunks = split_vert!(
      hor_chunks[0],
      1,
      [
        Constraint::Length(5),
        Constraint::Percentage(80),
        Constraint::Min(7),
      ]
    );
    let help_box = InfoBox::new(
      "Help",
      styled_block(vec![
        vec![
          (None, "Use "),
          (HIGHLIGHT, "tab "),
          (None, "to switch between new group input and group list"),
        ],
        vec![
          (None, "Pressing "),
          (HIGHLIGHT, "enter "),
          (None, "on an existing group will delete it."),
        ],
        vec![
          (None, "Adding the '"),
          (HIGHLIGHT, "wheel"),
          (None, "' group enables the use of "),
          (HIGHLIGHT, "sudo"),
          (None, "."),
        ],
      ]),
    );
    self.group_name_input.render(f, line_editor_chunks[0]);
    help_box.render(f, line_editor_chunks[2]);
    self.group_list.render(f, hor_chunks[1]);
  }
  pub fn handle_input_main_menu(
    &mut self,
    installer: &mut super::Installer,
    event: ratatui::crossterm::event::KeyEvent,
  ) -> Signal {
    if self.confirming_delete && event.code != KeyCode::Enter {
      self.confirming_delete = false;
      let buttons = vec![
        Box::new(Button::new("Change username")) as Box<dyn ConfigWidget>,
        Box::new(Button::new("Change password")) as Box<dyn ConfigWidget>,
        Box::new(Button::new("Edit Groups")) as Box<dyn ConfigWidget>,
        Box::new(Button::new("Delete user")) as Box<dyn ConfigWidget>,
      ];
      self.buttons.set_children_inplace(buttons);
    }
    match event.code {
      ui_down!() => {
        if !self.buttons.next_child() {
          self.buttons.first_child();
        }
        Signal::Wait
      }
      ui_up!() => {
        if !self.buttons.prev_child() {
          self.buttons.last_child();
        }
        Signal::Wait
      }
      KeyCode::Enter => {
        match self.buttons.selected_child() {
          Some(0) => {
            // Change username
            self.buttons.unfocus();
            self.name_input.focus();
            Signal::Wait
          }
          Some(1) => {
            // Change password
            self.buttons.unfocus();
            self.pass_input.focus();
            Signal::Wait
          }
          Some(2) => {
            // Change shell
            self.buttons.unfocus();
            // preselect current shell
            if self.selected_user < installer.users.len() {
              let cur = installer.users[self.selected_user].shell.clone();
              // Start from the first item to avoid looping forever
              self.shell_list.first_item();
              // Scan through the list until it matches the current shell
              // (limit the scan to avoid infinite loops if items change)
              let mut scanned = 0usize;
              // If you have a len() on StrList, prefer that; else keep a small cap
              while self.shell_list.selected_item() != Some(&cur) && scanned < 8 {
                if !self.shell_list.next_item() { break; }
                scanned += 1;
              }
            }
            self.shell_list.focus();
            Signal::Wait
          }
          Some(3) => {
            // Edit groups
            self.buttons.unfocus();
            self.group_name_input.focus();
            Signal::Wait
          }
          Some(4) => {
            // Delete user
            if !self.confirming_delete {
              self.confirming_delete = true;
              let buttons = vec![
                Box::new(Button::new("Change username")) as Box<dyn ConfigWidget>,
                Box::new(Button::new("Change password")) as Box<dyn ConfigWidget>,
                Box::new(Button::new("Edit Groups")) as Box<dyn ConfigWidget>,
                Box::new(Button::new("Really?")) as Box<dyn ConfigWidget>,
              ];
              self.buttons.set_children_inplace(buttons);
              Signal::Wait
            } else {
              if self.selected_user < installer.users.len() {
                installer.users.remove(self.selected_user);
              }
              Signal::Pop
            }
          }
          _ => Signal::Wait,
        }
      }
      ui_back!() => Signal::Pop,
      _ => Signal::Wait,
    }
  }
  pub fn handle_input_name_change(
    &mut self,
    installer: &mut super::Installer,
    event: ratatui::crossterm::event::KeyEvent,
  ) -> Signal {
    match event.code {
      KeyCode::Enter => {
        // Get owned string, then trim
        let entered_owned = self
          .name_input
          .get_value()
          .and_then(|s| s.as_str().map(|s| s.to_owned()))
          .unwrap_or_default();

        let normalized = match normalize_and_validate_username(&entered_owned) {
          Ok(n) => n,
          Err(msg) => { self.name_input.error(msg); return Signal::Wait; }
        };

        // If unchanged, just go back to menu
        if self.selected_user < installer.users.len()
          && installer.users[self.selected_user].username == normalized
        {
          self.name_input.unfocus();
          self.buttons.focus();
          return Signal::Wait;
        }

        // Duplicate check against all *other* users
        if installer
          .users
          .iter()
          .enumerate()
          .any(|(i, u)| i != self.selected_user && u.username == normalized)
        {
          self.name_input.error("User already exists");
          return Signal::Wait;
        }

        // Apply rename
        if self.selected_user < installer.users.len() {
          installer.users[self.selected_user].username = normalized;
        }
        self.name_input.unfocus();
        self.buttons.focus();
        Signal::Wait
      }
      ui_close!() => {
        self.name_input.unfocus();
        self.buttons.focus();
        Signal::Wait
      }
      _ => self.name_input.handle_input(event),
    }
  }
  pub fn handle_input_pass_change(
    &mut self,
    installer: &mut super::Installer,
    event: ratatui::crossterm::event::KeyEvent,
  ) -> Signal {
    if self.pass_input.is_focused() {
      match event.code {
        KeyCode::Tab => {
          self.pass_input.clear_error();
          self.pass_input.unfocus();
          self.pass_confirm.focus();
          Signal::Wait
        }
        KeyCode::Enter => {
          if let Some(pass) = self.pass_input.get_value() {
            let Some(pass) = pass.as_str() else {
              self.pass_input.error("Password cannot be empty");
              return Signal::Wait;
            };
            if pass.is_empty() {
              self.pass_input.error("Password cannot be empty");
              return Signal::Wait;
            }
            self.pass_input.clear_error();
            self.pass_input.unfocus();
            self.pass_confirm.focus();
            Signal::Wait
          } else {
            self.pass_input.error("Password cannot be empty");
            Signal::Wait
          }
        }
        KeyCode::Esc => {
          self.pass_input.unfocus();
          self.buttons.focus();
          Signal::Wait
        }
        _ => self.pass_input.handle_input(event),
      }
    } else if self.pass_confirm.is_focused() {
      match event.code {
        KeyCode::Tab => {
          self.pass_confirm.unfocus();
          self.pass_input.focus();
          Signal::Wait
        }
        KeyCode::Esc => {
          self.pass_confirm.unfocus();
          self.buttons.focus();
          Signal::Wait
        }
        KeyCode::Enter => {
          if let Some(pass) = self.pass_input.get_value() {
            let Some(pass) = pass.as_str() else {
              self.pass_input.error("Password cannot be empty");
              return Signal::Wait;
            };
            if let Some(confirm) = self.pass_confirm.get_value() {
              let Some(confirm) = confirm.as_str() else {
                self
                  .pass_confirm
                  .error("Password confirmation cannot be empty");
                return Signal::Wait;
              };
              if pass != confirm {
                self.pass_confirm.clear();
                self.pass_confirm.error("Passwords do not match");
                self.pass_input.unfocus();
                self.pass_confirm.focus();
                return Signal::Wait;
              }
              let hashed = match super::RootPassword::mkpasswd(pass.to_string()) {
                Ok(h) => h,
                Err(e) => {
                  return Signal::Error(anyhow::anyhow!("Failed to hash password: {e}"));
                }
              };
              if self.selected_user < installer.users.len() {
                installer.users[self.selected_user].password_hash = hashed;
              }
              self.pass_confirm.unfocus();
              self.buttons.focus();
              Signal::Wait
            } else {
              self
                .pass_confirm
                .error("Password confirmation cannot be empty");
              Signal::Wait
            }
          } else {
            self.pass_input.error("Password cannot be empty");
            Signal::Wait
          }
        }
        _ => self.pass_confirm.handle_input(event),
      }
    } else {
      self.pass_input.focus();
      Signal::Wait
    }
  }
  pub fn handle_input_edit_groups(
    &mut self,
    installer: &mut super::Installer,
    event: ratatui::crossterm::event::KeyEvent,
  ) -> Signal {
    if self.group_name_input.is_focused() {
      match event.code {
        KeyCode::Enter => {
          let entered_owned = self
            .group_name_input
            .get_value()
            .and_then(|s| s.as_str().map(|s| s.to_owned()))
            .unwrap_or_default();
          let entered = entered_owned.trim().to_string();

          if entered.is_empty() {
            // User chose not to add any group.
            if self.groups_only {
              // Inline flow after AddUser: finish immediately → back to summary
              return Signal::PopCount(2);
            } else {
              // Editing via full AlterUser: go back to the menu quietly
              self.group_name_input.unfocus();
              self.buttons.focus();
              return Signal::Wait;
            }
          }

          // Non-empty: try to add the group
          if self.selected_user < installer.users.len() {
            let user = &mut installer.users[self.selected_user];

            if user.groups.iter().any(|g| g == &entered) {
              self.group_name_input.error("User already in group");
              return Signal::Wait;
            }

            user.groups.push(entered);
            // reflect in UI and clear input
            self.group_list.set_items(user.groups.clone());
            self.group_name_input.clear();
          }
          Signal::Wait
        }
        KeyCode::Tab => {
          if !self.group_list.is_empty() {
            self.group_name_input.unfocus();
            self.group_list.focus();
          }
          Signal::Wait
        }
        KeyCode::Esc => {
          if self.groups_only {
            return Signal::PopCount(2);
          }
          self.group_name_input.unfocus();
          self.buttons.focus();
          Signal::Wait
        }
        _ => self.group_name_input.handle_input(event),
      }
    } else if self.group_list.is_focused() {
      // Enter deletes items from the list
      match event.code {
        ui_down!() => {
          if !self.group_list.next_item() {
            self.group_list.first_item();
          }
          Signal::Wait
        }
        ui_up!() => {
          if !self.group_list.previous_item() {
            self.group_list.last_item();
          }
          Signal::Wait
        }
        KeyCode::Enter => {
          if let Some(selected) = self.group_list.selected_item()
            && self.selected_user < installer.users.len() {
              let user = &mut installer.users[self.selected_user];
              user.groups.retain(|g| g != selected);
              self.group_list.set_items(user.groups.clone());
            }

          if self.group_list.is_empty() {
            self.group_list.unfocus();
            self.group_name_input.focus();
          }
          Signal::Wait
        }
        KeyCode::Tab => {
          self.group_list.unfocus();
          self.group_name_input.focus();
          Signal::Wait
        }
        ui_close!() => {
          if self.groups_only {
            return Signal::PopCount(2);
          }
          self.group_list.unfocus();
          self.buttons.focus();
          Signal::Wait
        }
        _ => Signal::Wait,
      }
    } else {
      self.group_name_input.focus();
      Signal::Wait
    }
  }
}

impl Page for AlterUser {
  fn render(
    &mut self,
    installer: &mut super::Installer,
    f: &mut ratatui::Frame,
    area: ratatui::prelude::Rect,
  ) {
    if self.buttons.is_focused() {
      self.render_main_menu(f, area);
    } else if self.name_input.is_focused() {
      self.render_name_change(f, area);
    } else if self.pass_input.is_focused() || self.pass_confirm.is_focused() {
      self.render_pass_change(f, area);
    } else if self.group_name_input.is_focused() || self.group_list.is_focused() {
      self.render_edit_groups(installer, f, area);
    } else if self.shell_list.is_focused() {
      self.render_select_shell(f, area);  
    } else {
      self.buttons.focus();
      self.render_main_menu(f, area);
    }

    // Render help modal on top
    self.help_modal.render(f, area);
  }

  fn handle_input(
    &mut self,
    installer: &mut super::Installer,
    event: ratatui::crossterm::event::KeyEvent,
  ) -> Signal {
    match event.code {
      KeyCode::Char('?') => {
        self.help_modal.toggle();
        return Signal::Wait;
      }
      ui_close!() if self.help_modal.visible => {
        self.help_modal.hide();
        return Signal::Wait;
      }
      _ if self.help_modal.visible => {
        return Signal::Wait;
      }
      _ => {}
    }
  
    if self.buttons.is_focused() {
      self.handle_input_main_menu(installer, event)
    } else if self.name_input.is_focused() {
      self.handle_input_name_change(installer, event)
    } else if self.pass_input.is_focused() || self.pass_confirm.is_focused() {
      self.handle_input_pass_change(installer, event)
    } else if self.group_name_input.is_focused() || self.group_list.is_focused() {
      self.handle_input_edit_groups(installer, event)
    } else if self.shell_list.is_focused() {
      match event.code {
        ui_down!() => {
          if !self.shell_list.next_item() {
            // optional wrap-around; remove if you don't want wrapping
            self.shell_list.first_item();
          }
          Signal::Wait
        }
        ui_up!() => {
          if !self.shell_list.previous_item() {
            // optional wrap-around; remove if you don't want wrapping
            self.shell_list.last_item();
          }
          Signal::Wait
        }
        KeyCode::Esc | KeyCode::Tab | KeyCode::BackTab => {
          self.shell_list.unfocus();
          self.buttons.focus();
          Signal::Wait
        }
        KeyCode::Enter => {
          if let Some(sel) = self.shell_list.selected_item()
            && self.selected_user < installer.users.len() {
              installer.users[self.selected_user].shell = sel.to_string();
            }
          self.shell_list.unfocus();
          self.buttons.focus();
          Signal::Wait
        }
        _ => Signal::Wait,
      }
    } else {
      self.buttons.focus();
      Signal::Wait
    }
  }

  fn get_help_content(&self) -> (String, Vec<Line<'_>>) {
    let help_content = styled_block(vec![
      vec![
        (
          Some((
            ratatui::style::Color::Yellow,
            ratatui::style::Modifier::BOLD,
          )),
          "↑/↓, j/k",
        ),
        (None, " - Navigate menu options"),
      ],
      vec![
        (
          Some((
            ratatui::style::Color::Yellow,
            ratatui::style::Modifier::BOLD,
          )),
          "Enter, →, l",
        ),
        (None, " - Select option"),
      ],
      vec![
        (
          Some((
            ratatui::style::Color::Yellow,
            ratatui::style::Modifier::BOLD,
          )),
          "Tab",
        ),
        (None, " - Navigate between fields"),
      ],
      vec![
        (
          Some((
            ratatui::style::Color::Yellow,
            ratatui::style::Modifier::BOLD,
          )),
          "Esc, q, ←, h",
        ),
        (None, " - Return to previous menu"),
      ],
      vec![
        (
          Some((
            ratatui::style::Color::Yellow,
            ratatui::style::Modifier::BOLD,
          )),
          "?",
        ),
        (None, " - Show this help"),
      ],
      vec![(None, "")],
      vec![(
        None,
        "Modify an existing user account. Choose from changing",
      )],
      vec![(None, "username, password, groups, or deleting the user.")],
    ]);
    ("Alter User".to_string(), help_content)
  }
}