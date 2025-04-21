use yew::prelude::*;
use web_sys::HtmlInputElement;
use serde::{Serialize, Deserialize};
use gloo_storage::{LocalStorage, Storage};
use uuid::Uuid;

const STORAGE_KEY: &str = "todos";

const BUTTON_CLASS: &str = "px-2 py-1 rounded text-white";
const SAVE_BUTTON: &str = "ml-2 bg-green-500 hover:bg-green-600";
const CANCEL_BUTTON: &str = "ml-2 bg-gray-500 hover:bg-gray-600";
const EDIT_BUTTON: &str = "ml-2 bg-yellow-500 hover:bg-yellow-600";
const DELETE_BUTTON: &str = "ml-2 bg-red-500 hover:bg-red-600";
const ADD_BUTTON: &str = "bg-blue-500 hover:bg-blue-600 px-4 py-2 rounded";

#[derive(Clone, PartialEq, Serialize, Deserialize)]
struct Todo {
    id: String,
    title: String,
    completed: bool,
}

fn create_new_todo(todos: &[Todo], title: String) -> Vec<Todo> {
    let mut new_todos = Vec::with_capacity(todos.len() + 1);
    new_todos.extend(todos.iter().cloned());
    new_todos.push(Todo {
        id: Uuid::new_v4().to_string(),
        title,
        completed: false,
    });
    new_todos
}

fn is_valid_title(title: &str) -> bool {
    !title.is_empty()
}

fn read_input_title(input: &HtmlInputElement) -> String {
    input.value().trim().to_string()
}

fn save_todos_to_storage_with_error(
    key: &str,
    todos: &[Todo],
    error_handle: &UseStateHandle<Option<String>>,
) {
    if let Err(e) = LocalStorage::set(key, todos) {
        error_handle.set(Some(format!("Storage error: {:?}", e)));
    } else {
        error_handle.set(None);
    }
}

fn update_todos_state(todos_handle: &UseStateHandle<Vec<Todo>>, new_todos: Vec<Todo>) {
    todos_handle.set(new_todos);
}

fn update_todos(
    todos_handle: &UseStateHandle<Vec<Todo>>,
    new_todos: Vec<Todo>,
    error_handle: &UseStateHandle<Option<String>>,
) {
    save_todos_to_storage_with_error(STORAGE_KEY, &new_todos, error_handle);
    update_todos_state(todos_handle, new_todos);
}

fn clear_input(input: &HtmlInputElement) {
    input.set_value("");
}

fn delete_todo(todos: &[Todo], id: &str) -> Vec<Todo> {
    todos.iter().filter(|todo| todo.id != id).cloned().collect()
}

fn toggle_todo(todos: &[Todo], id: &str) -> Vec<Todo> {
    todos
        .iter()
        .map(|todo| {
            if todo.id == id {
                Todo {
                    completed: !todo.completed,
                    ..todo.clone()
                }
            } else {
                todo.clone()
            }
        })
        .collect()
}

fn update_todo_title(todos: &[Todo], id: &str, title: &str) -> Vec<Todo> {
    todos
        .iter()
        .map(|todo| {
            if todo.id == id {
                Todo {
                    title: title.to_string(),
                    ..todo.clone()
                }
            } else {
                todo.clone()
            }
        })
        .collect()
}

fn clear_edit_state(edit_id_handle: &UseStateHandle<Option<String>>) {
    edit_id_handle.set(None);
}

fn set_edit_state(edit_id_handle: &UseStateHandle<Option<String>>, id: &str) {
    edit_id_handle.set(Some(id.to_string()));
}

fn focus_input(input_ref: &NodeRef) {
    if let Some(input) = input_ref.cast::<HtmlInputElement>() {
        if input.focus().is_err() {
            web_sys::console::log_1(&"Failed to focus input".into());
        }
    }
}

#[function_component(App)]
fn app() -> Html {
    let storage_error = use_state(|| None::<String>);
    let todos = use_state(|| {
        match LocalStorage::get(STORAGE_KEY) {
            Ok(todos) => todos,
            Err(e) => {
                storage_error.set(Some(format!("Failed to load todos: {:?}", e)));
                Vec::<Todo>::new()
            }
        }
    });

    let input_ref = use_node_ref();
    let edit_id = use_state(|| None::<String>);
    let edit_input_ref = use_node_ref();

    let on_submit = {
        let todos = todos.clone();
        let input_ref = input_ref.clone();
        let storage_error = storage_error.clone();
        Callback::from(move |e: SubmitEvent| {
            e.prevent_default();
            if let Some(input) = input_ref.cast::<HtmlInputElement>() {
                let title = read_input_title(&input);
                if is_valid_title(&title) {
                    let new_todos = create_new_todo(&todos, title);
                    update_todos(&todos, new_todos, &storage_error);
                    clear_input(&input);
                }
            }
        })
    };

    let on_delete = {
        let todos = todos.clone();
        let storage_error = storage_error.clone();
        Callback::from(move |id: String| {
            let new_todos = delete_todo(&todos, &id);
            update_todos(&todos, new_todos, &storage_error);
        })
    };

    let on_toggle = {
        let todos = todos.clone();
        let storage_error = storage_error.clone();
        Callback::from(move |id: String| {
            let new_todos = toggle_todo(&todos, &id);
            update_todos(&todos, new_todos, &storage_error);
        })
    };

    let on_edit = {
        let edit_id = edit_id.clone();
        let edit_input_ref = edit_input_ref.clone();
        Callback::from(move |id: String| {
            set_edit_state(&edit_id, &id);
            focus_input(&edit_input_ref);
        })
    };

    let on_update = {
        let todos = todos.clone();
        let edit_id = edit_id.clone();
        let edit_input_ref = edit_input_ref.clone();
        let storage_error = storage_error.clone();
        Callback::from(move |id: String| {
            if let Some(input) = edit_input_ref.cast::<HtmlInputElement>() {
                let title = read_input_title(&input);
                if is_valid_title(&title) {
                    let new_todos = update_todo_title(&todos, &id, &title);
                    update_todos(&todos, new_todos, &storage_error);
                    clear_edit_state(&edit_id);
                }
            }
        })
    };

    let on_cancel = {
        let edit_id = edit_id.clone();
        Callback::from(move |_| clear_edit_state(&edit_id))
    };

    let render_todo = |id: String, title: String, completed: bool, is_editing: bool| {
        let id_for_toggle = id.clone();
        let id_for_edit = id.clone();
        let id_for_delete = id;
        html! {
            <li class="flex items-center p-2 border rounded">
                if is_editing {
                    <input
                        type="text"
                        ref={edit_input_ref.clone()}
                        value={title}
                        class="flex-grow p-1 border rounded focus:outline-none focus:ring-2 focus:ring-blue-500"
                    />
                    <button
                        onclick={on_update.reform(move |_| id_for_edit.clone())}
                        class={format!("{} {}", BUTTON_CLASS, SAVE_BUTTON)}
                    >
                        {"Save"}
                    </button>
                    <button
                        onclick={on_cancel.clone()}
                        class={format!("{} {}", BUTTON_CLASS, CANCEL_BUTTON)}
                    >
                        {"Cancel"}
                    </button>
                } else {
                    <input
                        type="checkbox"
                        checked={completed}
                        onclick={on_toggle.reform(move |_| id_for_toggle.clone())}
                        class="mr-2"
                    />
                    <span class={if completed { "line-through flex-grow" } else { "flex-grow" }}>
                        { title }
                    </span>
                    <button
                        onclick={on_edit.reform(move |_| id_for_edit.clone())}
                        class={format!("{} {}", BUTTON_CLASS, EDIT_BUTTON)}
                    >
                        {"Edit"}
                    </button>
                    <button
                        onclick={on_delete.reform(move |_| id_for_delete.clone())}
                        class={format!("{} {}", BUTTON_CLASS, DELETE_BUTTON)}
                    >
                        {"Delete"}
                    </button>
                }
            </li>
        }
    };

    html! {
        <div class="container mx-auto p-4 max-w-md">
            <h1 class="text-2xl font-bold mb-4 text-center">{"Todo App"}</h1>
            <form onsubmit={on_submit} class="mb-4">
                <div class="flex gap-2">
                    <input
                        type="text"
                        ref={input_ref}
                        placeholder="Add a new task"
                        class="flex-grow p-2 border rounded focus:outline-none focus:ring-2 focus:ring-blue-500"
                    />
                    <button
                        type="submit"
                        class={ADD_BUTTON}
                    >
                        {"Add"}
                    </button>
                </div>
            </form>
            {
                (*storage_error).as_ref().map_or_else(
                    || html! {},
                    |error| html! { <p class="text-red-500">{ error }</p> }
                )
            }
            <ul class="space-y-2">
                { for (*todos).iter().map(|todo| {
                    let is_editing = edit_id.as_ref() == Some(&todo.id);
                    render_todo(todo.id.clone(), todo.title.clone(), todo.completed, is_editing)
                })}
            </ul>
        </div>
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_add_new_todo_to_existing_list() {
        let todos = vec![Todo {
            id: "1".to_string(),
            title: "Create Yew + TW + Rust App".to_string(),
            completed: false,
        }];
        let new_todos = create_new_todo(&todos, "New Task".to_string());
        assert_eq!(new_todos.len(), 2);
        assert_eq!(new_todos[1].title, "New Task");
        assert_eq!(new_todos[1].completed, false);
    }

    #[test]
    fn should_validate_non_empty_title() {
        assert_eq!(is_valid_title("Welcom Rust"), true);
    }

    #[test]
    fn should_invalidate_empty_or_whitespace_title() {
        assert_eq!(is_valid_title(""), false);
        assert_eq!(is_valid_title("  "), false);
    }

    #[test]
    fn should_remove_todo_by_id() {
        let todos = vec![
            Todo {
                id: "1".to_string(),
                title: "Task 1".to_string(),
                completed: false,
            },
            Todo {
                id: "2".to_string(),
                title: "Task 2".to_string(),
                completed: true,
            },
        ];
        let new_todos = delete_todo(&todos, "1");
        assert_eq!(new_todos.len(), 1);
        assert_eq!(new_todos[0].id, "2");
        assert_eq!(new_todos[0].title, "Task 2");
        assert_eq!(new_todos[0].completed, true);
    }

    #[test]
    fn should_toggle_todo_completion_status() {
        let todos = vec![
            Todo {
                id: "1".to_string(),
                title: "Task 1".to_string(),
                completed: false,
            },
            Todo {
                id: "2".to_string(),
                title: "Task 2".to_string(),
                completed: true,
            },
        ];
        let new_todos = toggle_todo(&todos, "1");
        assert_eq!(new_todos.len(), 2);
        assert_eq!(new_todos[0].id, "1");
        assert_eq!(new_todos[0].title, "Task 1");
        assert_eq!(new_todos[0].completed, true);
        assert_eq!(new_todos[1].id, "2");
        assert_eq!(new_todos[1].title, "Task 2");
        assert_eq!(new_todos[1].completed, true);
    }

    #[test]
    fn should_update_todo_title_by_id() {
        let todos = vec![
            Todo {
                id: "1".to_string(),
                title: "Task 1".to_string(),
                completed: false,
            },
            Todo {
                id: "2".to_string(),
                title: "Task 2".to_string(),
                completed: true,
            },
        ];
        let new_todos = update_todo_title(&todos, "1", "Updated Task");
        assert_eq!(new_todos.len(), 2);
        assert_eq!(new_todos[0].id, "1");
        assert_eq!(new_todos[0].title, "Updated Task");
        assert_eq!(new_todos[0].completed, false);
        assert_eq!(new_todos[1].id, "2");
        assert_eq!(new_todos[1].title, "Task 2");
        assert_eq!(new_todos[1].completed, true);
    }
}

fn main() {
    yew::Renderer::<App>::new().render();
}