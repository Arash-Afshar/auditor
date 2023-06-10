use std::collections::HashSet;

use crate::{AllInfo, LatestFileInfo, LatestFileInfos, StoredReviewForFile};
use leptos::{ev::MouseEvent, *};
// use leptos_meta::*;
// use leptos_router::*;

#[component]
pub fn App(cx: Scope) -> impl IntoView {
    // provide_meta_context(cx);
    //view! {
    //    cx,
    //    <Stylesheet id="leptos" href="/pkg/tailwind.css"/>
    //    <Link rel="shortcut icon" type_="image/ico" href="/favicon.ico"/>
    //    <Router>
    //        <Routes>
    //            <Route path="" view=  move |cx| view! { cx, <Home/> }/>
    //        </Routes>
    //    </Router>
    //}

    view! {
        cx,
        <Home/>
    }
}

#[component]
fn CaretTop(cx: Scope) -> impl IntoView {
    view! {
        cx,
        <svg data-accordion-icon class="w-6 h-6 rotate-180 shrink-0" fill="currentColor" viewBox="0 0 20 20" xmlns="http://www.w3.org/2000/svg"><path fill-rule="evenodd" d="M5.293 7.293a1 1 0 011.414 0L10 10.586l3.293-3.293a1 1 0 111.414 1.414l-4 4a1 1 0 01-1.414 0l-4-4a1 1 0 010-1.414z" clip-rule="evenodd"></path></svg>
    }
}

#[component]
fn CaretDown(cx: Scope) -> impl IntoView {
    view! {
        cx,
        <svg data-accordion-icon class="w-6 h-6 shrink-0" fill="currentColor" viewBox="0 0 20 20" xmlns="http://www.w3.org/2000/svg"><path fill-rule="evenodd" d="M5.293 7.293a1 1 0 011.414 0L10 10.586l3.293-3.293a1 1 0 111.414 1.414l-4 4a1 1 0 01-1.414 0l-4-4a1 1 0 010-1.414z" clip-rule="evenodd"></path></svg>

    }
}

#[component]
fn AccordionButton<F1, F2>(
    cx: Scope,
    file_name: String,
    line_info: StoredReviewForFile,
    has_comments: bool,
    is_first: bool,
    expanded: F1,
    on_click: F2,
) -> impl IntoView
where
    F1: Fn() -> bool,
    F2: Fn(MouseEvent) + 'static,
{
    let truncate = |name: String| {
        let limit = 20;
        if name.len() <= limit {
            name
        } else {
            format!("...{}", name[name.len() - limit..].to_string())
        }
    };

    view! {
        cx,
        <div class="flex flex-row gap-5  border border-gray-200 dark:border-gray-700" class=("rounded-t-xl", move || is_first)>
            <div class="ml-5 flex flex-grow items-center gap-5 text-left text-gray-500 dark:text-gray-400 font-medium">
                <div>{truncate(file_name)}</div>
                <div class="text-blue-500">{if has_comments {"yes"} else {"no"}}</div>
                <div class="text-green-500">{line_info.percent_reviewed()}<span class="font-thin text-xs">" %"</span></div>
                <div class="text-red-600">{line_info.percent_modified()}<span class="font-thin text-xs">" %"</span></div>
                <div class="text-gray-400">{line_info.percent_ignored()}<span class="font-thin text-xs">" %"</span></div>
            </div>
            <div>
                <button
                    type="button"
                    class="flex items-center justify-between w-full p-5 font-medium text-left text-gray-500 focus:ring-4 focus:ring-gray-200 dark:focus:ring-gray-800 dark:text-gray-400 hover:bg-gray-100 dark:hover:bg-gray-800"
                    class=("rounded-t-xl", move || is_first)
                    on:click=on_click
                >
                    {if expanded() {
                        view!{cx, <CaretDown/>}.into_view(cx)
                    } else {
                        view!{cx, <CaretTop/>}.into_view(cx)
                    }}
                </button>
            </div>
        </div>
    }
}

#[component]
fn SearchBar(cx: Scope) -> impl IntoView {
    view! {
        cx,
        <div class="m-6">
            <input
                type="text"
                id="file_name"
                class="bg-gray-50 border border-gray-300 text-gray-900 text-sm rounded-lg focus:ring-blue-500 focus:border-blue-500 block w-full p-2.5 dark:bg-gray-700 dark:border-gray-600 dark:placeholder-gray-400 dark:text-white dark:focus:ring-blue-500 dark:focus:border-blue-500"
                placeholder="Search for file names" required
            />
        </div>
    }
}

#[component]
fn ExpandableComment<F>(
    cx: Scope,
    file_name: String,
    file_info: LatestFileInfo,
    is_first: bool,
    expanded: ReadSignal<HashSet<String>>,
    on_click: F,
) -> impl IntoView
where
    F: Fn(MouseEvent) + 'static,
{
    let id = format!("heading-{}", &file_name);
    let file_name_clone = file_name.clone();
    let expanded = Signal::derive(cx, move || expanded().contains(&file_name_clone));

    let has_comments = !file_info.comments.is_empty();
    let display = move || {
        if file_info.comments.is_empty() {
            view! {
                cx,
                 <p class="text-gray-500 dark:text-gray-400">
                  "No comments!"
                 </p>
            }
            .into_view(cx)
        } else {
            let contents = file_info
                .comments
                .clone()
                .into_iter()
                .map(|(line_number, content)| {
                    view! {
                        cx,
                        <div class="flex flex-row gap-10">
                            <div class="min-w-[50px]">{format!("line#{}", line_number)}</div>
                            <div class="flex-grow text-left">
                                {content.iter().map(|comment| {
                                    view!{
                                        cx,
                                        <div class="flex flex-row gap-5">
                                            <div class="min-w-[100px]">{format!("-{}:", comment.author.clone())}</div>
                                            <div>{comment.body.clone()}</div>
                                        </div>
                                    }
                                }).collect_view(cx)}
                            </div>
                        </div>
                    }
                    .into_view(cx)
                })
                .collect_view(cx);
            view! {
                cx,
                <div class="flex flex-col gap-3 text-gray-500 dark:text-gray-400">
                    {contents}
                </div>
            }
            .into_view(cx)
        }
    };

    view! {
        cx,
        <div id>
            <AccordionButton file_name={file_name.clone()} line_info={file_info.line_reviews} has_comments is_first expanded on_click/>
        </div>
        <div class=("hidden", move || !expanded()) aria-labelledby={id}>
            <div class="p-5 border border-gray-200 dark:border-gray-700 dark:bg-gray-900">
                {display}
            </div>
        </div>
    }
}

#[component]
fn Comments(cx: Scope, info: AllInfo) -> impl IntoView {
    // Contains the list of files that all are expanded
    let (expanded, set_expanded) = create_signal(cx, HashSet::<String>::default());
    if !info.file_info.is_empty() {
        let keys = Vec::from_iter(info.file_info.keys());
        let first_file_name = keys.first().unwrap();
        set_expanded.update(|set| {
            set.insert(first_file_name.to_string());
        });
    }

    let info = info
        .file_info
        .into_iter()
        .enumerate()
        .map(|(idx, (file_name, file_info))| {
            view! {
                cx,
                <ExpandableComment
                    file_name={file_name.clone()}
                    file_info
                    is_first={idx==0}
                    expanded
                    on_click=move |_| {
                        if expanded().contains(&file_name) {
                            set_expanded.update(|set| {set.remove(&file_name);})
                        } else {
                            set_expanded.update(|set| {set.insert(file_name.clone());})
                        }
                    }
                />
            }
        })
        .collect_view(cx);

    view! {
        cx,
        <div>
            {info}
        </div>
    }
}

#[component]
fn Home(cx: Scope) -> impl IntoView {
    let asyc_comments = create_resource(
        cx,
        || (),
        |_| async move {
            log!("loading data from API");
            let request_url = "http://localhost:3000/info";
            let response = reqwest::get(request_url).await.unwrap();
            log!("Got response");

            let all_info: LatestFileInfos = response.json().await.unwrap();
            log!("Converted response to comments");
            all_info
        },
    );

    fn transform(info: LatestFileInfos) -> AllInfo {
        AllInfo { file_info: info.0 }
    }

    view! { cx,
        <div class="my-0 text-center min-h-screen min-w-full dark:bg-gray-950">
            <div class="container-xl  mx-auto max-w-3xl ">
                <h2 class="p-6 text-4xl dark:text-gray-100">"Review Report"</h2>
                <SearchBar/>
                <div class="m-5">
                    {move || match asyc_comments.read(cx) {
                        None => view! { cx, <p>"Loading..."</p> }.into_view(cx),
                        Some(comments) => view! { cx, <Comments info={transform(comments)}/> }.into_view(cx)
                    }}
                </div>
            </div>
        </div>
    }
}

// #[component]
// fn Home(cx: Scope) -> impl IntoView {
//     let (toggled, set_toggled) = create_signal(cx, HashSet::<usize>::default());
//     view! { cx,
//         <div>{move || toggled.get().contains(&4)}</div>
//         <ButtonB on_click=move |_| set_toggled.update(|value| {value.insert(4);})/>
//     }
// }
// #[component]
// pub fn ButtonB<F>(cx: Scope, on_click: F) -> impl IntoView
// where
//     F: Fn(MouseEvent) + 'static,
// {
//     view! { cx,
//         <button on:click=on_click>
//             "Toggle"
//         </button>
//     }
// }
