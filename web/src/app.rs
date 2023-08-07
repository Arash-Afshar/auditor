use std::collections::HashSet;

use crate::{
    Filters, LatestFileInfo, LatestFileInfos, Metadata, Priority, PriorityBF, StoredReviewForFile,
    UpdateMetadataRequest,
};
use leptos::html::{Input, Select};
use leptos::{
    ev::{MouseEvent, SubmitEvent},
    *,
};
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
    full_file_name: String,
    line_info: StoredReviewForFile,
    comments_count: usize,
    metadata: Option<Metadata>,
    is_first: bool,
    expanded: F1,
    on_click: F2,
) -> impl IntoView
where
    F1: Fn() -> bool,
    F2: Fn(MouseEvent) + 'static,
{
    let file_name: Vec<&str> = full_file_name.split("/").collect();
    let file_name = file_name.last().unwrap().to_string();
    let note = metadata.clone().map(|m| m.note).unwrap_or("".to_string());

    view! {
        cx,
        <div class="flex flex-row gap-5  border border-gray-200 dark:border-gray-700" class=("rounded-t-xl", move || is_first)>
            <div class="ml-5 flex flex-grow items-center gap-5 text-left text-gray-500 dark:text-gray-400 font-medium"  >
                <div class="min-w-[70px]"></div>
                {
                    match metadata {
                        Some(metadata) => match metadata.priority {
                            Priority::Unspecified => view!{cx, <div class="text-blue-600 min-w-[40px]">{metadata.reviewer}</div>},
                            Priority::High => view!{cx, <div class="text-red-600 min-w-[40px]">{metadata.reviewer}</div>},
                            Priority::Medium => view!{cx, <div class="text-yellow-400 min-w-[40px]">{metadata.reviewer}</div>},
                            Priority::Low => view!{cx, <div class="text-green-500 min-w-[40px]">{metadata.reviewer}</div>},
                            Priority::Ignore => view!{cx, <div class="text-gray-600 min-w-[40px]">{metadata.reviewer}</div>},
                        },
                        None => view!{cx, <div>"no metadata!"</div>}
                    }
                }
                <div class="min-w-[120px]">{file_name}</div>

                <div class="flex-grow text-black">{note}</div>

                <div class="text-blue-500 min-w-[40px]">{format!("({comments_count})")}</div>
                <div class="text-green-500 min-w-[40px]">{line_info.percent_reviewed()}<span class="font-thin text-xs">" %"</span></div>
                <div class="text-red-600 min-w-[40px]">{line_info.percent_modified()}<span class="font-thin text-xs">" %"</span></div>
                <div class="text-gray-400 min-w-[40px]">{line_info.percent_ignored()}<span class="font-thin text-xs">" %"</span></div>
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
fn SearchBar(cx: Scope, search: RwSignal<String>) -> impl IntoView {
    view! {
        cx,
        <div class="m-6">
            <input
                type="text"
                id="file_name"
                class="bg-gray-50 border border-gray-300 text-gray-900 text-sm rounded-lg focus:ring-blue-500 focus:border-blue-500 block w-full p-2.5 dark:bg-gray-700 dark:border-gray-600 dark:placeholder-gray-400 dark:text-white dark:focus:ring-blue-500 dark:focus:border-blue-500"
                placeholder="Search for file names" required
                on:input=move |ev| search.update(|f| *f = event_target_value(&ev))
            />

        </div>
    }
}

async fn update_metadata(update_metadata_request: &UpdateMetadataRequest) -> String {
    let client = reqwest::Client::new();
    match client
        .post("http://localhost:3000/metadata")
        .json(update_metadata_request)
        .send()
        .await
    {
        Ok(_response) => "Saved!".to_string(),
        Err(e) => e.to_string(),
    }
}

#[component]
fn FileDetails(
    cx: Scope,
    reviewers: Vec<String>,
    full_file_name: String,
    metadata: Option<Metadata>
) -> impl IntoView {
    let reviewer = metadata.unwrap().reviewer;

    // we'll use a NodeRefs to store references to the input elements
    // these will be filled when the elements are created
    let note_element: NodeRef<Input> = create_node_ref(cx);
    let priority_element: NodeRef<Select> = create_node_ref(cx);
    let reviewer_element: NodeRef<Select> = create_node_ref(cx);

    let update_metadata_action = create_action(cx, |request: &UpdateMetadataRequest| {
        let request = request.to_owned();
        async move { update_metadata(&request).await }
    });

    let on_submit = move |ev: SubmitEvent| {
        ev.prevent_default(); // stop the page from reloading!

        let note = note_element().expect("<input> to exist").value();
        let priority_str = priority_element().expect("<select> to exist").value();
        // let reviewer = reviewer_element().expect("<select> to exist").value();
        let reviewer = reviewer.clone();

        let request = UpdateMetadataRequest {
            file_name: full_file_name.clone(),
            metadata: Metadata {
                priority: priority_str.parse().unwrap(),
                reviewer: reviewer, // TODO: unchanged reviewer for now
                note: note,
            },
        };

        update_metadata_action.dispatch(request);
    };

    // let reviewers = ["Yi-Hsiu", "Arash"]; // Hardcore the list of reviewers here.

    view! { cx,
        <form on:submit=on_submit>
            <b>"Note: "</b>
            <input type="text"
                node_ref=note_element
                placeholder="old note will be override"
                class="bg-gray-50 border border-gray-300 text-gray-900 text-sm rounded-lg focus:ring-blue-500 focus:border-blue-500 p-2.5 dark:bg-gray-700 dark:border-gray-600 dark:placeholder-gray-400 dark:text-white dark:focus:ring-blue-500 dark:focus:border-blue-500"
            />
            <b>"　Priority: "</b>
            <select id="priority" name="priority" node_ref=priority_element>
                <option value="Unspecified">"Unspecified"</option>
                <option value="Ignore">"Ignore"</option>
                <option value="Low">"Low"</option>
                <option value="Medium">"Medium"</option>
                <option value="High">"High"</option>
            </select>
            <b>"　Reviewer: "</b>
            <select id="reviewer" name="reviewer" node_ref=reviewer_element>
                <option value="Unassigned">"Unassigned"</option>
                {reviewers.into_iter()
                    .map(|reviewer| view! { cx, <option value={&reviewer}>{reviewer}</option>})
                    .collect::<Vec<_>>()}
            </select>
            <b>"　　"</b>
            <input type="submit" value="Save" class="font-medium focus:ring-4 focus:ring-gray-200 dark:focus:ring-gray-800 dark:text-gray-400 hover:bg-gray-100 dark:hover:bg-gray-800"/>
            "　"{update_metadata_action.value()}
        </form>
    }
}

#[component]
fn ExpandableComment<F>(
    cx: Scope,
    reviewers: Vec<String>,
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
    let file_name_clone_2 = file_name.clone();
    let expanded = Signal::derive(cx, move || expanded().contains(&file_name_clone));

    let comments_count = file_info.comments.len();
    let metadata: Option<crate::Metadata> = file_info.metadata;
    let display = move || {
        if file_info.comments.is_empty() {
            view! {
                cx,
                <div class="text-gray-500 dark:text-gray-400">
                    <p>
                    "No comments!"
                    </p>
                    <p class="text-left">{format!("full path: {}", &file_name_clone_2)}</p>
                </div>
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
                <div class="flex flex-col gap-3 text-gray-500 dark:text-gray-400 text-left">
                    {contents}
                    <p>{format!("full path: {}", &file_name_clone_2)}</p>
                </div>
            }
            .into_view(cx)
        }
    };

    view! {
        cx,
        <div id>
            <AccordionButton full_file_name={file_name.clone()} line_info={file_info.line_reviews} comments_count metadata=metadata.clone() is_first expanded on_click/>
        </div>
        <div class=("hidden", move || !expanded()) aria-labelledby={&id}>
            <FileDetails full_file_name={file_name.clone()} metadata={metadata.clone()} reviewers={reviewers.clone()}/>
        </div>
        <div class=("hidden", move || !expanded()) aria-labelledby={&id}>
            <div class="p-5 border border-gray-200 dark:border-gray-700 dark:bg-gray-900">
                {display}
            </div>
        </div>
    }
}

#[component]
fn Comments(cx: Scope, info: LatestFileInfos, reviewers: Vec<String>) -> impl IntoView {
    // Contains the list of files that all are expanded
    let (expanded, set_expanded) = create_signal(cx, HashSet::<String>::default());
    if !info.0.is_empty() {
        let first_file_name = info.0.first().unwrap();
        set_expanded.update(|set| {
            set.insert(first_file_name.file_name.clone());
        });
    }

    let info_view = info
        .clone()
        .0
        .into_iter()
        .enumerate()
        .map(|(idx, file_info)| {
            let file_name = file_info.file_name.clone();
            view! {
                cx,
                <ExpandableComment
                    reviewers={reviewers.clone()}
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

    let file_count = move || info.0.len();

    view! {
        cx,
        <div>
            <div class="dark:text-gray-100 text-left">"File count:"</div>
            <div class="dark:text-gray-100 text-left">{file_count}</div>
            {info_view}
        </div>
    }
}

#[component]
fn FiltersView(cx: Scope, filters: RwSignal<Filters>) -> impl IntoView {
    let sort_by_modified = move || filters().sort_by_modified;
    let sort_by_reviewed = move || filters().sort_by_reviewed;
    let sort_by_name = move || filters().sort_by_name;

    let filter_checkbox_class_str = "w-4 h-4 text-blue-600 bg-gray-100 border-gray-300 rounded focus:ring-blue-500 dark:focus:ring-blue-600 dark:ring-offset-gray-800 focus:ring-2 dark:bg-gray-700 dark:border-gray-600";
    let filter_label_class_str = "ml-2 text-sm font-medium dark:text-gray-300";

    view! {
        cx,
        <div class="flex flex-col gap-5 m-6 p-5 text-left dark:text-gray-100 border border-gray-300 dark:border-gray-600 rounded-lg">
            <div class="flex flex-row gap-5">
                <p><b>"By filetype"</b></p>
                <div class="flex items-center">
                    <input checked={move || filters().only_with_comments} on:change=move |ev| filters.update(|f| f.only_with_comments = event_target_checked(&ev)) id="comments_only" type="checkbox" value="" class="w-4 h-4 text-blue-600 bg-gray-100 border-gray-300 rounded focus:ring-blue-500 dark:focus:ring-blue-600 dark:ring-offset-gray-800 focus:ring-2 dark:bg-gray-700 dark:border-gray-600"/>
                    <label for="comments_only" class="ml-2 text-sm font-medium text-gray-900 dark:text-gray-300">"Comments"</label>
                </div>
                <div class="flex items-center">
                    <input checked={move || filters().only_c_files} on:change=move |ev| filters.update(|f| f.only_c_files = event_target_checked(&ev)) id="cpp" type="checkbox" value="" class="w-4 h-4 text-blue-600 bg-gray-100 border-gray-300 rounded focus:ring-blue-500 dark:focus:ring-blue-600 dark:ring-offset-gray-800 focus:ring-2 dark:bg-gray-700 dark:border-gray-600"/>
                    <label for="cpp" class="ml-2 text-sm font-medium text-gray-900 dark:text-gray-300">"C"</label>
                </div>
                <div class="flex items-center">
                    <input checked={move || filters().only_go_files} on:change=move |ev| filters.update(|f| f.only_go_files = event_target_checked(&ev)) id="go" type="checkbox" value="" class="w-4 h-4 text-blue-600 bg-gray-100 border-gray-300 rounded focus:ring-blue-500 dark:focus:ring-blue-600 dark:ring-offset-gray-800 focus:ring-2 dark:bg-gray-700 dark:border-gray-600"/>
                    <label for="go" class="ml-2 text-sm font-medium text-gray-900 dark:text-gray-300">"Go"</label>
                </div>
            </div>
            <div class="flex flex-row gap-5">
                <p><b>"By reviewer"</b></p>
                <div class="flex items-center">
                    <input checked={move || filters().reviewer_unassigned} on:change=move |ev| filters.update(|f| f.reviewer_unassigned = event_target_checked(&ev)) id="unassigned" type="checkbox" value="" class={filter_checkbox_class_str}/>
                    <label for="unassigned" class={filter_label_class_str}>"Unassigned"</label>
                </div>
            </div>
            <div class="flex flex-row gap-5">
                <p><b>"By priority"</b></p>
                <div class="flex items-center">
                    <input checked={move || filters().priority_mask.contains(PriorityBF::UNSPECIFIED)} on:change=move |ev| filters.update(|f| f.priority_mask.set(PriorityBF::UNSPECIFIED, event_target_checked(&ev))) id="prior_unspecified" type="checkbox" value="" class={filter_checkbox_class_str}/>
                    <label for="prior_unspecified" class={filter_label_class_str.to_owned()+" text-blue-600"}>"Unspecified"</label>
                </div>
                <div class="flex items-center">
                    <input checked={move || filters().priority_mask.contains(PriorityBF::HIGH)} on:change=move |ev| filters.update(|f| f.priority_mask.set(PriorityBF::HIGH, event_target_checked(&ev))) id="prior_high" type="checkbox" value="" class={filter_checkbox_class_str}/>
                    <label for="prior_high" class={filter_label_class_str.to_owned()+" text-red-600"}>"High"</label>
                </div>
                <div class="flex items-center">
                    <input checked={move || filters().priority_mask.contains(PriorityBF::MEDIUM)} on:change=move |ev| filters.update(|f| f.priority_mask.set(PriorityBF::MEDIUM, event_target_checked(&ev))) id="prior_medium" type="checkbox" value="" class={filter_checkbox_class_str}/>
                    <label for="prior_medium" class={filter_label_class_str.to_owned()+" text-yellow-400"}>"Medium"</label>
                </div>
                <div class="flex items-center">
                    <input checked={move || filters().priority_mask.contains(PriorityBF::LOW)} on:change=move |ev| filters.update(|f| f.priority_mask.set(PriorityBF::LOW, event_target_checked(&ev))) id="prior_low" type="checkbox" value="" class={filter_checkbox_class_str}/>
                    <label for="prior_low" class={filter_label_class_str.to_owned()+" text-green-500"}>"Low"</label>
                </div>
                <div class="flex items-center">
                    <input checked={move || filters().priority_mask.contains(PriorityBF::IGNORE)} on:change=move |ev| filters.update(|f| f.priority_mask.set(PriorityBF::IGNORE, event_target_checked(&ev))) id="prior_ignore" type="checkbox" value="" class={filter_checkbox_class_str}/>
                    <label for="prior_ignore" class={filter_label_class_str.to_owned()+" text-gray-600"}>"Ignore"</label>
                </div>
            </div>
            <div class="flex flex-row gap-5">
                <div class="flex items-center">
                    <input checked={sort_by_modified} on:change=move |ev| filters.update(|f| {
                        f.sort_by_modified = event_target_checked(&ev);
                        if f.sort_by_modified {
                            f.sort_by_name = false;
                            f.sort_by_reviewed = false;
                        }
                    })
                    id="sort-by-modified"
                    name="sort"
                    type="radio"
                    class="w-4 h-4 text-blue-600 bg-gray-100 border-gray-300 rounded focus:ring-blue-500 dark:focus:ring-blue-600 dark:ring-offset-gray-800 focus:ring-2 dark:bg-gray-700 dark:border-gray-600"/>
                    <label for="sort-by-modified" class="ml-2 text-sm font-medium text-gray-900 dark:text-gray-300">"Sort by modified"</label>
                </div>
                <div class="flex items-center">
                    <input checked={sort_by_reviewed} on:change=move |ev| filters.update(|f| {
                        f.sort_by_reviewed = event_target_checked(&ev);
                        if f.sort_by_reviewed {
                            f.sort_by_name = false;
                            f.sort_by_modified = false;
                        }
                    })
                    id="sort-by-reviewed"
                    name="sort"
                    type="radio"
                    class="w-4 h-4 text-blue-600 bg-gray-100 border-gray-300 rounded focus:ring-blue-500 dark:focus:ring-blue-600 dark:ring-offset-gray-800 focus:ring-2 dark:bg-gray-700 dark:border-gray-600"/>
                    <label for="sort-by-reviewed" class="ml-2 text-sm font-medium text-gray-900 dark:text-gray-300">"Sort by reviewed"</label>
                </div>
                <div class="flex items-center">
                    <input checked={sort_by_name} on:change=move |ev| filters.update(|f| {
                        f.sort_by_name = event_target_checked(&ev);
                        if f.sort_by_name {
                            f.sort_by_reviewed = false;
                            f.sort_by_modified = false;
                        }
                    })
                    id="sort-by-name"
                    name="sort"
                    type="radio"
                    class="w-4 h-4 text-blue-600 bg-gray-100 border-gray-300 rounded focus:ring-blue-500 dark:focus:ring-blue-600 dark:ring-offset-gray-800 focus:ring-2 dark:bg-gray-700 dark:border-gray-600"/>
                    <label for="sort-by-name" class="ml-2 text-sm font-medium text-gray-900 dark:text-gray-300">"Sort by name"</label>
                </div>
            </div>
        </div>
    }
}

#[component]
fn Home(cx: Scope) -> impl IntoView {
    let filters = create_rw_signal(cx, Filters::default());
    let search = create_rw_signal(cx, String::new());

    let asyc_comments = create_resource(
        cx,
        || (),
        |_| async move {
            let request_url = "http://localhost:3000/info";
            let response = reqwest::get(request_url).await.unwrap();

            let all_info: LatestFileInfos = response.json().await.unwrap();

            let reviewers: HashSet<_> = all_info
                .0
                .iter()
                .map(|info: &LatestFileInfo| match &info.metadata {
                    Some(Metadata {
                        priority: _,
                        reviewer,
                        note: _,
                    }) => reviewer.clone(),
                    None => "Unassigned".to_string(),
                })
                .collect();

            let mut reviewers: Vec<String> = reviewers.iter().cloned().collect();
            reviewers.sort();

            (all_info, reviewers)
        },
    );

    let filter = move |info: LatestFileInfos| {
        let mut filtered: Vec<LatestFileInfo> = info
            .0
            .into_iter()
            .filter(|info| {
                if !search().is_empty() {
                    if !info.file_name.contains(&search()) {
                        return false;
                    }
                }
                if let Some(Metadata {
                    priority,
                    reviewer,
                    note: _,
                }) = &info.metadata
                {
                    let priority = match priority {
                        Priority::Unspecified => PriorityBF::UNSPECIFIED,
                        Priority::High => PriorityBF::HIGH,
                        Priority::Medium => PriorityBF::MEDIUM,
                        Priority::Low => PriorityBF::LOW,
                        Priority::Ignore => PriorityBF::IGNORE,
                    };
                    if reviewer == "Unassigned" && !filters().reviewer_unassigned {
                        return false;
                    }
                    if !filters().priority_mask.contains(priority) {
                        return false;
                    }
                    return true;
                }
                if info.line_reviews.percent_ignored() == 100 {
                    return false;
                }
                let file_name = info.file_name.clone();
                if filters().only_with_comments && info.comments.is_empty() {
                    return false;
                }
                if filters().only_c_files && filters().only_go_files {
                    if !(file_name.ends_with(".c")
                        || file_name.ends_with(".cpp")
                        || file_name.ends_with(".h")
                        || file_name.ends_with(".go"))
                    {
                        return false;
                    }
                } else if filters().only_c_files
                    && !(file_name.ends_with(".c")
                        || file_name.ends_with(".cpp")
                        || file_name.ends_with(".h"))
                {
                    return false;
                } else if filters().only_go_files && !file_name.ends_with(".go") {
                    return false;
                }
                true
            })
            .collect();

        if filters().sort_by_name {
            filtered.sort_by(|a, b| a.file_name.partial_cmp(&b.file_name).unwrap());
        } else if filters().sort_by_modified {
            filtered.sort_by(|a, b| {
                b.line_reviews
                    .percent_modified()
                    .partial_cmp(&a.line_reviews.percent_modified())
                    .unwrap()
            });
        } else if filters().sort_by_reviewed {
            filtered.sort_by(|a, b| {
                a.line_reviews
                    .percent_reviewed()
                    .partial_cmp(&b.line_reviews.percent_reviewed())
                    .unwrap()
            });
        }
        LatestFileInfos(filtered)
    };

    view! { cx,
        <div class="pb-40 my-0 text-center min-h-screen min-w-full dark:bg-gray-950">
            <div class="container-xl  mx-auto max-w-3xl ">
                <h2 class="p-6 text-4xl dark:text-gray-100">"Review Report"</h2>
                <SearchBar search/>
                <FiltersView filters />
                <div class="m-5">
                    {move || match asyc_comments.read(cx) {
                        None => view! { cx, <p>"Loading..."</p> }.into_view(cx),
                        Some(resource) => view! {cx, <Comments
                            info={filter(resource.0)}
                            reviewers=resource.1/> }.into_view(cx)
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
