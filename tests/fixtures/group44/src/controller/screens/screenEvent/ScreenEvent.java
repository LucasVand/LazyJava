package controller.screens.screenEvent;

/**
 * This enum represents different screen events within Party Islands.
 * These events will be posted through {@link ScreenEventBus}.
 * 
 * @author Sam Deitz
 * @see controller.screens.screenEvent.ScreenEventBus
 */
public enum ScreenEvent {
    GO_TO_MAIN_MENU,
    GO_TO_LOGIN,
    GO_TO_ROOMS,
    JOIN_ROOM,
    GO_TO_HIGH_SCORES,
    GO_TO_INSTRUCTIONS,
    GO_TO_ADMIN_LOGIN,
    GO_TO_ADMIN_CONTROLS,
    GO_TO_PLAYER_SCREEN,
    GO_TO_CREATE_ACCOUNT
}
