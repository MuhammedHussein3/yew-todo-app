use yew::{prelude::*, virtual_dom::Key};
use web_sys::HtmlInputElement;
use serde::{Serialize, Deserialize};
use gloo_storage::{LocalStorage, Storage};
use uuid::Uuid;

const STORAGE_KEY: &str = "todos";

#[derive(Clone, PartialEq, Serialize, Deserialize)]
struct Todo {
    id: String,
    title: String,
    completed: bool,
}

fn create_new_todo(todos: &[Todo], title: String) -> Vec<Todo> {
    let mut new_todos = todos.to_vec();
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

fn save_todos_to_storage(key: &str, todos: &[Todo]) {
    if let Err(e) = LocalStorage::set(key, todos) {
        web_sys::console::log_1(&format!("Storage error: {:?}", e).into());
    }
}

fn update_todos_state(todos_handle: &UseStateHandle<Vec<Todo>>, new_todos: Vec<Todo>) {
    todos_handle.set(new_todos);
}

fn clear_input(input: &HtmlInputElement) {
    input.set_value("");
}

fn delete_todo(todos: &[Todo], id: &str) -> Vec<Todo> {
    todos.iter().filter(|todo| todo.id != id).cloned().collect()
}

fn toggle_todo(todos: &[Todo], id: &str) -> Vec<Todo> {
    todos.iter().filter(|todo| todo.id != id).cloned().collect()
}

fn update_todo_title(todos: &[Todo], id: &str, title: &str) -> Vec<Todo> {
    let mut new_todos = todos.to_vec();
    if let Some(todo) = new_todos.iter_mut().find(|todo| todo.id == id) {
        todo.title = title.to_string();
    }

    new_todos
}

fn clear_edit_state(edit_id_handle: &UseStateHandle<Option<String>>) {
    edit_id_handle.set(None);
}

fn set_edit_state(edit_id_handle: &UseStateHandle<Option<String>>, id: &str) {
    edit_id_handle.set(Some(id.to_string()));
}

fn focus_input(input_ref: &NodeRef) {
    if let Some(input) = input_ref.cast::<HtmlInputElement>() {
        input.focus().unwrap();
    }
}


#[function_component(App)]
fn app() -> Html {
    let todos = use_state(|| {
        LocalStorage::get(STORAGE_KEY).unwrap_or_else(|_| Vec::<Todo>::new())
    });

    let input_ref = use_node_ref();
    let edit_id = use_state(|| None::<String>);
    let edit_input_ref = use_node_ref();

    // Cruds: First Project (Welcome Rust)

    let on_submit = {
        let todos = todos.clone();
        let input_ref = input_ref.clone();
        Callback::from(move |e: SubmitEvent | {
            e.prevent_default();
            if let Some(input) = input_ref.cast::<HtmlInputElement>() {
                let title = read_input_title(&input);
                
                if is_valid_title(&title) {
                    let new_todos = create_new_todo(&todos, title);

                    save_todos_to_storage(STORAGE_KEY, &new_todos);

                    update_todos_state(&todos, new_todos);
                    
                    clear_input(&input);
                }
            }
        })
    };

    let on_delete = {
        let todos = todos.clone();
        Callback::from(move |id: String| {

            let new_todos = delete_todo(&todos, &id);

            save_todos_to_storage(STORAGE_KEY, &new_todos);

            update_todos_state(&todos, new_todos);
        })
    };

    let on_toggle = {
        let todos = todos.clone();
        Callback::from(move |id: String| {

            let new_todos = toggle_todo(&todos, &id);

            save_todos_to_storage(STORAGE_KEY, &new_todos);

            update_todos_state(&todos, new_todos);
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
        Callback::from(move |id: String| {
            if let Some(input) = edit_input_ref.cast::<HtmlInputElement>() {
                let title = read_input_title(&input);

                if is_valid_title(&title) {
                    let new_todos = update_todo_title(&todos, &id, &title);
                    
                    save_todos_to_storage(STORAGE_KEY, &new_todos);

                    update_todos_state(&todos, new_todos);

                    clear_edit_state(&edit_id);
                }
            }
        })
    };


    // Cancel Edit
    let on_cancel = {
        let edit_id = edit_id.clone();
        Callback::from(move |_| edit_id.set(None))
    };

    // Function to render a single todo item
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
                        class="ml-2 bg-green-500 text-white px-2 py-1 rounded hover:bg-green-600"
                    >
                        {"Save"}
                    </button>
                    <button
                        onclick={on_cancel.clone()}
                        class="ml-2 bg-gray-500 text-white px-2 py-1 rounded hover:bg-gray-600"
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
                        class="ml-2 bg-yellow-500 text-white px-2 py-1 rounded hover:bg-yellow-600"
                    >
                        {"Edit"}
                    </button>
                    <button
                        onclick={on_delete.reform(move |_| id_for_delete.clone())}
                        class="ml-2 bg-red-500 text-white px-2 py-1 rounded hover:bg-red-600"
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
                        class="bg-blue-500 text-white px-4 py-2 rounded hover:bg-blue-600"
                    >
                        {"Add"}
                    </button>
                </div>
            </form>
            <ul class="space-y-2">
                { for (*todos).iter().map(|todo| {
                    let is_editing = edit_id.as_ref() == Some(&todo.id);
                    render_todo(todo.id.clone(), todo.title.clone(), todo.completed, is_editing)
                })}
            </ul>
        </div>
    }
}

fn main() {
    yew::Renderer::<App>::new().render();
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