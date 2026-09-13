const APP_SOURCE: &str = include_str!("../../src/App.vue");
const HOME_VIEW_SOURCE: &str = include_str!("../../src/components/HomeView.vue");
const HOME_CONTEXT_SOURCE: &str = include_str!("../../src/components/viewContexts.ts");
const HOME_CONTEXT_BUILDER_SOURCE: &str = include_str!("../../src/composables/useHomeViewContext.ts");
const GOAL_SETTING_SOURCE: &str = include_str!("../../src/composables/useHeatmapGoalSetting.ts");

#[test]
fn home_goal_editor_is_wired_to_the_persisted_goal_setting() {
    assert!(
        GOAL_SETTING_SOURCE.contains("function setHeatmapGoalMinutes(minutes: number)"),
        "goal setting composable must expose an explicit user-edit path"
    );
    assert!(
        GOAL_SETTING_SOURCE.contains("schedulePersistHeatmapGoal();"),
        "user goal edits must keep the existing debounced persistence path"
    );
    assert!(
        HOME_CONTEXT_SOURCE.contains("learnGoalMinutes: number;")
            && HOME_CONTEXT_SOURCE.contains("setLearnGoalMinutes: (minutes: number) => void;"),
        "Home context must expose both the current goal and its mutation handler"
    );
    assert!(
        HOME_CONTEXT_BUILDER_SOURCE.contains("learnGoalMinutes: learnGoalMinutes.value")
            && HOME_CONTEXT_BUILDER_SOURCE.contains("setLearnGoalMinutes,"),
        "Home context builder must pass the editable goal through"
    );
    assert!(
        APP_SOURCE.contains("setHeatmapGoalMinutes,")
            && APP_SOURCE.contains("learnGoalMinutes: learnGoalSliderMinutes")
            && APP_SOURCE.contains("setLearnGoalMinutes: setHeatmapGoalMinutes"),
        "App wiring must connect the persisted setting composable to Home"
    );
    assert!(
        HOME_VIEW_SOURCE.contains("id=\"home-goal-minutes\"")
            && HOME_VIEW_SOURCE.contains("type=\"range\"")
            && HOME_VIEW_SOURCE.contains("min=\"0\"")
            && HOME_VIEW_SOURCE.contains("max=\"1440\"")
            && HOME_VIEW_SOURCE.contains("step=\"15\"")
            && HOME_VIEW_SOURCE.contains("@input=\"handleGoalSliderInput\""),
        "Home must render an interactive 0-24h goal editor with the existing 15-minute step"
    );
}

#[test]
fn reading_goal_seconds_does_not_schedule_a_write() {
    let marker = "function getHeatmapGoalSeconds(): number {";
    let start = GOAL_SETTING_SOURCE
        .find(marker)
        .expect("getHeatmapGoalSeconds function");
    let function_source = &GOAL_SETTING_SOURCE[start..];
    let end = function_source
        .find("\n  }\n\n")
        .expect("getHeatmapGoalSeconds function end");
    let body = &function_source[..end];

    assert!(
        !body.contains("schedulePersistHeatmapGoal") && !body.contains("syncGoalFromSlider"),
        "reading the goal for Home/Insights refreshes must not enqueue persistence writes"
    );
}
