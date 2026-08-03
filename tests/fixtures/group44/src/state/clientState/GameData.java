package state.clientState;

/**
 * GameData, this is a placeholder interface for anything that wants to be used
 * as data in a {@code ClientState}, has no requirements but helps to show
 * intent when looking at the class implementation
 */
public interface GameData {
    /**
     * called when the object is supposed to be dropped and destroyed, removes all
     * listeners and resets any lingering effects
     */
    public void drop();

    /**
     * Sets the on state change callback which is called when any state is changed
     * the ui needs to rerender
     *
     * @param callback the callback which is going to be ran when the ui needs to
     *                 update
     */
    public void setOnStateChange(Runnable callback);

    /**
     * calls the on state change which updates the ui
     */
    void updateState();
}
