/**
MIT License

Copyright (c) 2017 Leppsoft

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE
 */


import java.io.FileNotFoundException;
import java.io.IOException;
import java.io.RandomAccessFile;
import java.nio.charset.Charset;
import java.nio.charset.StandardCharsets;

enum PlayStatus {
	STOPPED, PLAYING, PAUSED, SEEKING
}

public class SoundpadRemoteControl {
	public static final String CLIENT_VERSION = "1.1.1";
	private static final Charset CHARSET_UTF8 = StandardCharsets.UTF_8;
	private final boolean PRINT_ERRORS = true;
	private long lastRequestTimestamp = System.currentTimeMillis();
	private RandomAccessFile pipe = null;

	/**
	 * You may optionally call this method to handle the exception. This method
	 * is also internally called whenever a request is sent.
	 *
	 * @throws FileNotFoundException
	 *             if communication cannot be established.
	 */
	public void init() throws FileNotFoundException {
		if (pipe == null) {
			pipe = new RandomAccessFile("\\\\.\\pipe\\sp_remote_control", "rw");
		}
	}

	public void uninit() {
		if (pipe != null) {
			try {
				pipe.close();
			} catch (IOException ignore) {
			} finally {
				pipe = null;
			}
		}
	}

	/**
	 * Sends a command to Soundpad. Depending on the command the result contains
	 * a status code or a message.
	 *
	 * @param request
	 *            command to be executed by Soundpad
	 * @return string response or empty string if transmission fails. The
	 *         response is based on HTTP status codes, e.g. R-200 for OK or
	 *         R-404 for Not found. Certain commands get a custom response, e.g.
	 *         {@link #getSoundlist()} returns an xml formatted sound list.
	 * @throws IOException
	 *             if communication fails
	 */
	private synchronized String sendRequest(final String request) throws IOException {
		init();

		if (System.currentTimeMillis() == lastRequestTimestamp) {
			// doing too many requests at the same time can break the pipe
			try {
				Thread.sleep(1);
			} catch (InterruptedException ignore) {
			}
		}

		pipe.write(request.getBytes());

		// data size in pipe is first acquirable after reading one byte
		byte firstByte = pipe.readByte();
		byte[] responseBytes = new byte[(int) pipe.length() + 1];
		responseBytes[0] = firstByte;
		pipe.readFully(responseBytes, 1, responseBytes.length - 1);

		lastRequestTimestamp = System.currentTimeMillis();

		return new String(responseBytes, CHARSET_UTF8);
	}

	/**
	 * @see #sendRequest(String)
	 * @param request
	 *            command to be executed by Soundpad
	 * @return response or empty string if transmission fails.
	 */
	private synchronized String sendRequestNoException(final String request) {
		try {
			return sendRequest(request);
		} catch (IOException e) {
			uninit();
		}
		return "";
	}

	/**
	 * Helper method to print consolidated errors.
	 */
	private void printOfflineError() {
		if (PRINT_ERRORS) {
			System.err.println("Remote control is offline.");
		}
	}

	/**
	 * @see #printOfflineError()
	 */
	private void printNumericError(final String response) {
		if (PRINT_ERRORS) {
			System.err.println("Expected numeric response, but received: " + response);
		}
	}

	/**
	 * @see #printOfflineError()
	 */
	private void printError(final String response) {
		if (PRINT_ERRORS) {
			System.err.println("Failed: " + response);
		}
	}

	private boolean isSuccess(final String response) {
		return response.startsWith("R-200");
	}

	private boolean isSuccessPrintResponse(final String response) {
		if (!response.equals("R-200")) {
			printError(response);
			return false;
		}
		return true;
	}

	private long handleNumericLongGetRequest(final String request) {
		String response = sendRequestNoException(request);
		if (response.startsWith("R")) {
			printError(response);
		} else if (response.isEmpty()) {
			printOfflineError();
		} else {
			try {
				return Long.parseLong(response);
			} catch (NumberFormatException e) {
				printNumericError(response);
			}
		}

		return -1;
	}

	private String handleStringGetRequest(final String request) {
		String response = sendRequestNoException(request);
		if (response.startsWith("R")) {
			printError(response);
		} else if (response.isEmpty()) {
			printOfflineError();
		}
		return response;
	}

	private String handleSimpleGetRequest(final String request) {
		String response = sendRequestNoException(request);
		if (response.startsWith("R")) {
			printError(response);
		}
		return response;
	}

	private String handleEmptyGetRequest(final String request) {
		String response = sendRequestNoException(request);
		if (response.isEmpty()) {
			printOfflineError();
		}

		return response;
	}

	//***********************************************************************//
	//
	// Remote Control v1.0.0
	//
	// Methods which were added in the first version of the interface.
	//
	//***********************************************************************//

	/**
	 * Let Soundpad play the sound at the given index. Index is the first column
	 * in the sound list and is tied to the <b>All sounds</b> category. Every
	 * sound has a unique index, which can be changed by moving the sound within
	 * the All sounds category. The All sounds category is hidden by default and
	 * can be accessed in Soundpad from the menu at Window > Categories > All
	 * sounds.
	 *
	 * @param index
	 *            Get the index by calling {@link #getSoundlist()} first.
	 * @return true if a sound with the given index exists.
	 */
	public boolean playSound(final int index) {
		return isSuccess(sendRequestNoException("DoPlaySound(" + index + ")"));
	}

	/**
	 * Extends {@link #playSound(int)} by the ability to determine on which
	 * lines the sound shall be played.
	 *
	 * @param index
	 *            Get the index by calling {@link #getSoundlist()} first.
	 * @param renderLine
	 *            set to true to play on speakers so you hear it.
	 * @param captureLine
	 *            set to true to play on microphone so others hear it.
	 * @return true on success
	 */
	public boolean playSound(final int index, final boolean renderLine, final boolean captureLine) {
		String response = sendRequestNoException(
				"DoPlaySound(" + index + ", " + renderLine + ", " + captureLine + ")");
		return isSuccess(response);
	}

	/**
	 * Play previous sound in the list. The play mode is the same as it was for
	 * the last played file. Means, if a sound was played on speakers only, then
	 * this function will play on speakers only as well.
	 *
	 * @return true on success
	 */
	public boolean playPreviousSound() {
		return isSuccess(sendRequestNoException("DoPlayPreviousSound()"));
	}

	/**
	 * @see #playPreviousSound()
	 * @return true on success
	 */
	public boolean playNextSound() {
		return isSuccess(sendRequestNoException("DoPlayNextSound()"));
	}

	public boolean stopSound() {
		return isSuccess(sendRequestNoException("DoStopSound()"));
	}

	public boolean togglePause() {
		return isSuccess(sendRequestNoException("DoTogglePause()"));
	}

	/**
	 * Use negative values to jump backwards.
	 *
	 * @param timeMillis
	 *            e.g. 5000 to jump 5 seconds forward.
	 */
	public boolean jump(final int timeMillis) {
		return isSuccess(sendRequestNoException("DoJumpMs(" + timeMillis + ")"));
	}

	/**
	 * Jump to a particular position in the currently played sound.
	 *
	 * @param timeMillis
	 *            e.g. 5000 to jump to the 5th second of the sound.
	 */
	public boolean seek(final int timeMillis) {
		return isSuccess(sendRequestNoException("DoSeekMs(" + timeMillis + ")"));
	}

	/**
	 * Start recording. This call is handled the same way as if a recording is
	 * started by hotkeys, which means a notification sound is played. This is
	 * the default behavior, but the notification sound can be turned off in
	 * Soundpad.
	 *
	 * @return true if recording was started or was already running
	 */
	public boolean startRecording() {
		return isSuccess(sendRequestNoException("DoStartRecording()"));
	}

	public boolean stopRecording() {
		return isSuccess(sendRequestNoException("DoStopRecording()"));
	}

	/**
	 * Uses Soundpad's instant search to highlight sounds.
	 */
	public boolean search(final String searchTerm) {
		String response = sendRequestNoException("DoSearch(\"" + searchTerm + "\")");
		if (!response.equals("R-200")) {
			printError(response);
			return false;
		}
		return true;
	}

	/**
	 * Closes search panel.
	 */
	public boolean resetSearch() {
		return isSuccess(sendRequestNoException("DoResetSearch()"));
	}

	/**
	 * Select previous search hit. Search is always wrapped. Means it starts
	 * again at the first hit if the last hit is reached.
	 */
	public boolean selectPreviousHit() {
		return isSuccess(sendRequestNoException("DoSelectPreviousHit()"));
	}

	/**
	 * @see #selectPreviousHit()
	 */
	public boolean selectNextHit() {
		return isSuccess(sendRequestNoException("DoSelectNextHit()"));
	}

	/**
	 * Select the sound at the given row in the list. This method was created
	 * before categories were introduced, as such the row is not the sound
	 * index, but the position in the currently selected category.
	 */
	public boolean selectRow(final int row) {
		return isSuccess(sendRequestNoException("DoSelectIndex(" + row + ")"));
	}

	/**
	 * Scroll down or up by this many rows. Use negative values to scroll
	 * upwards.
	 */
	public boolean scrollBy(final int rows) {
		return isSuccess(sendRequestNoException("DoScrollBy(" + rows + ")"));
	}

	/**
	 * Scroll to a particular row.
	 */
	public boolean scrollTo(final int row) {
		return isSuccess(sendRequestNoException("DoScrollTo(" + row + ")"));
	}

	/**
	 * Returns the total amount of sounds over all categories independent of
	 * currently selected category.
	 */
	public long getSoundFileCount() {
		return handleNumericLongGetRequest("GetSoundFileCount()");
	}

	/**
	 * Returns playback position of currently played sound file in milliseconds.
	 */
	public long getPlaybackPosition() {
		return handleNumericLongGetRequest("GetPlaybackPositionInMs()");
	}

	/**
	 * Returns duration of currently played sound file in milliseconds.
	 */
	public long getPlaybackDuration() {
		return handleNumericLongGetRequest("GetPlaybackDurationInMs()");
	}

	/**
	 * Returns recording position in milliseconds.
	 */
	public long getRecordingPosition() {
		return handleNumericLongGetRequest("GetRecordingPositionInMs()");
	}

	/**
	 * Returns current recording peak.
	 */
	public long getRecordingPeak() {
		return handleNumericLongGetRequest("GetRecordingPeak()");
	}

	/**
	 * Get entire sound list. Will return the sounds from the <b>All sounds</b>
	 * category. The All sounds category is hidden by default and can be
	 * accessed in Soundpad from the menu at Window > Categories > All sounds.
	 *
	 * @return xml formatted sound list
	 */
	public String getSoundlist() {
		return handleStringGetRequest("GetSoundlist()");
	}

	/**
	 * Get a section of the sound list from the given index to the end.
	 *
	 * @see #getSoundlist()
	 *
	 * @param fromIndex
	 *            starts with 1
	 * @return xml formatted sound list
	 */
	public String getSoundlist(final int fromIndex) {
		return handleStringGetRequest("GetSoundlist(" + fromIndex + ")");
	}

	/**
	 * Get a section of the sound list.
	 *
	 * @see #getSoundlist()
	 *
	 * @param fromIndex
	 *            starts with 1
	 * @param toIndex
	 *            the sound file at toIndex is included in the response
	 * @return xml formatted sound list
	 */
	public String getSoundlist(final int fromIndex, final int toIndex) {
		return handleStringGetRequest("GetSoundlist(" + fromIndex + "," + toIndex + ")");
	}

	public String getMainFrameTitleText() {
		return handleSimpleGetRequest("GetTitleText()");
	}

	public String getStatusBarText() {
		return handleSimpleGetRequest("GetStatusBarText()");
	}

	public PlayStatus getPlayStatus() {
		String response = sendRequestNoException("GetPlayStatus()");
		if (response.startsWith("R")) {
			printError(response);
		}

		try {
			return PlayStatus.valueOf(response);
		} catch (IllegalArgumentException ignore) {
			return PlayStatus.STOPPED;
		}
	}

	/**
	 * Returns the version of Soundpad. Not the version of the remote control
	 * interface.
	 */
	public String getVersion() {
		return handleEmptyGetRequest("GetVersion()");

	}

	public String getRemoteControlVersion() {
		return handleEmptyGetRequest("GetRemoteControlVersion()");
	}

	/**
	 * Add sound at the end of the list.
	 *
	 * @param url
	 *            full path and file name, e.g. C:\mysounds\sound.mp3
	 * @return false if sound does not exist or input is malformed.
	 */
	public boolean addSound(final String url) {
		return isSuccessPrintResponse(sendRequestNoException("DoAddSound(\"" + url + "\")"));
	}

	/**
	 * Add sound at the given index to the list.
	 *
	 * @param url
	 *            full path and file name, e.g. C:\mysounds\sound.mp3
	 * @return false if sound does not exist or input is malformed.
	 */
	public boolean addSound(final String url, final int insertAtIndex) {
		return isSuccessPrintResponse(
				sendRequestNoException("DoAddSound(\"" + url + "\", " + insertAtIndex + ")"));
	}

	/**
	 * Removes selected sound file entries.
	 *
	 * @return true on success
	 */
	public boolean removeSelectedEntries() {
		return removeSelectedEntries(false);
	}

	/**
	 * @see #removeSelectedEntries()
	 *
	 * @param removeOnDiskToo
	 *            triggers confirmation dialog if true
	 */
	public boolean removeSelectedEntries(final boolean removeOnDiskToo) {
		return isSuccess(sendRequestNoException("DoRemoveSelectedEntries(" + removeOnDiskToo + ")"));
	}

	/**
	 * Undo last action. Same as Edit > Undo in Soundpad.
	 */
	public boolean undo() {
		return isSuccess(sendRequestNoException("DoUndo()"));
	}

	/**
	 * Redo last action. Same as Edit > Redo in Soundpad.
	 */
	public boolean redo() {
		return isSuccess(sendRequestNoException("DoRedo()"));
	}

	/**
	 * Shows file selection dialog if sound list was never saved before.
	 */
	public boolean saveSoundlist() {
		return isSuccess(sendRequestNoException("DoSaveSoundlist()"));
	}

	/**
	 * @return volume between 0 and 100.
	 */
	public int getVolume() {
		String response = sendRequestNoException("GetVolume()");
		if (response.isEmpty()) {
			printOfflineError();
		} else {
			try {
				return Integer.parseInt(response);
			} catch (NumberFormatException e) {
				printNumericError(response);
			}
		}
		return 0;
	}

	/**
	 * @return true if the volume of the speakers is 0 or muted.
	 */
	public boolean isMuted() {
		String response = sendRequestNoException("IsMuted()");
		if (response.isEmpty()) {
			printOfflineError();
		} else {
			try {
				return Integer.parseInt(response) == 1;
			} catch (NumberFormatException e) {
				printNumericError(response);
			}
		}
		return false;
	}

	/**
	 * Change volume of the speakers.
	 *
	 * @param volume
	 *            a value between 0 and 100.
	 */
	public boolean setVolume(final int volume) {
		return isSuccess(sendRequestNoException("SetVolume(" + volume + ")"));
	}

	/**
	 * Mutes or unmutes speakers in Soundpad.
	 */
	public boolean toggleMute() {
		return isSuccess(sendRequestNoException("DoToggleMute()"));
	}

	/**
	 * @return true if this client class uses the same remote control interface
	 *         version as Soundpad.
	 */
	public boolean isCompatible() {
		return CLIENT_VERSION.equals(getRemoteControlVersion());
	}

	/**
	 * @return true if Soundpad is running and the remote control interface is
	 *         accessible.
	 */
	public boolean isAlive() {
		return isSuccess(sendRequestNoException("IsAlive()"));
	}

	//***********************************************************************//
	//
	// Remote Control v1.1.0
	//
	//***********************************************************************//

	public boolean playSelectedSound() {
		return isSuccess(sendRequestNoException("DoPlaySelectedSound()"));
	}

	public boolean playCurrentSoundAgain() {
		return isSuccess(sendRequestNoException("DoPlayCurrentSoundAgain()"));
	}

	public boolean playPreviouslyPlayedSound() {
		return isSuccess(sendRequestNoException("DoPlayPreviouslyPlayedSound()"));
	}

	/**
	 * Add a category at the bottom of the category list.
	 *
	 * @param name
	 *            Name of the category, must not be empty.
	 * @return true on success
	 */
	public boolean addCategory(final String name) {
		return addCategory(name, -1);
	}

	/**
	 * Add a category.
	 *
	 * @param name
	 *            Name of the category, must not be empty.
	 * @param parentCategoryIndex
	 *            Set to -1 to add it at the bottom or set a category index to
	 *            add it at the bottom of that category. Use
	 *            {@link #getCategories(boolean, boolean)} to find the index of
	 *            a category.
	 * @return true on success
	 */
	public boolean addCategory(final String name, final int parentCategoryIndex) {
		return isSuccessPrintResponse(
				sendRequestNoException("DoAddCategory(\"" + name + "\", " + parentCategoryIndex + ")"));
	}

	/**
	 * Add sound to a particular category and position there-in.
	 *
	 * @param url
	 *            full path and file name, e.g. C:\mysounds\sound.mp3
	 * @return true on success
	 */
	public boolean addSound(final String url, final int categoryIndex, final int insertAtPosition) {
		return isSuccessPrintResponse(sendRequestNoException(
				"DoAddSound(\"" + url + "\", " + categoryIndex + ", " + insertAtPosition + ")"));
	}

	/**
	 * Start recording of the speakers. Method might fail if the microphone is
	 * currently being recorded. This call is handled the same way as if a
	 * recording is started by hotkeys, which means a notification sound is
	 * played. This is the default behavior, but the notification sound can be
	 * turned off in Soundpad.
	 *
	 * @return true if recording was started or was already running
	 */
	public boolean startRecordingSpeakers() {
		return isSuccessPrintResponse(sendRequestNoException("DoStartRecordingSpeakers()"));
	}

	/**
	 * Start recording of the microphone. Method might fail if the speakers are
	 * currently being recorded. This call is handled the same way as if a
	 * recording is started by hotkeys, which means a notification sound is
	 * played. This is the default behavior, but the notification sound can be
	 * turned off in Soundpad.
	 *
	 * @return true if recording was started or was already running
	 */
	public boolean startRecordingMicrophone() {
		return isSuccessPrintResponse(sendRequestNoException("DoStartRecordingMicrophone()"));
	}

	/**
	 * Select the category identified by its index. Use
	 * {@link #getCategories(boolean, boolean)} to get the index.
	 *
	 * @param categoryIndex
	 *            The index of the category to be selected.
	 * @return true on success
	 */
	public boolean selectCategory(final int categoryIndex) {
		return isSuccessPrintResponse(sendRequestNoException("DoSelectCategory(" + categoryIndex + ")"));
	}

	public boolean selectPreviousCategory() {
		return isSuccess(sendRequestNoException("DoSelectPreviousCategory()"));
	}

	public boolean selectNextCategory() {
		return isSuccess(sendRequestNoException("DoSelectNextCategory()"));
	}

	/**
	 * Remove a category identified by its index. Use
	 * {@link #getCategories(boolean, boolean)} to get the index.
	 *
	 * @param categoryIndex
	 *            The index of the category to be removed.
	 * @return true on success
	 */
	public boolean removeCategory(final int categoryIndex) {
		return isSuccessPrintResponse(sendRequestNoException("DoRemoveCategory(" + categoryIndex + ")"));
	}

	/**
	 * Get the category tree.
	 *
	 * @param withSounds
	 *            includes all sound entries of each category into the response
	 * @param withIcons
	 *            base64 encoded PNGs
	 * @return xml formatted category list
	 */
	public String getCategories(final boolean withSounds, final boolean withIcons) {
		String cmd = String.format("GetCategories(%b, %b)", withSounds, withIcons);
		String response = sendRequestNoException(cmd);
		if (response.startsWith("R")) {
			printError(response);
		} else if (response.isEmpty()) {
			printOfflineError();
		}
		return response;
	}

	/**
	 * Get a category identified by its index. Use
	 * {@link #getCategories(boolean, boolean)} to get the index.
	 *
	 * @param withSounds
	 *            includes all sound entries associated to that category
	 * @param withIcons
	 *            base64 encoded PNG
	 * @return xml formatted category list
	 */
	public String getCategory(final int categoryIndex, final boolean withSounds, final boolean withIcons) {
		String cmd = String.format("GetCategory(%d, %b, %b)", categoryIndex, withSounds, withIcons);
		String response = sendRequestNoException(cmd);
		if (response.startsWith("R")) {
			printError(response);
		} else if (response.isEmpty()) {
			printOfflineError();
		}
		return response;
	}

	//***********************************************************************//
	//
	// Remote Control v1.1.1
	//
	//***********************************************************************//

	/**
	 * Let Soundpad play a sound from a particular category.
	 *
	 * @param categoryIndex
	 *            set to -1 to play a sound from the currently selected category
	 *            or use {@link #getCategories(boolean, boolean)} to find the
	 *            index of a category.
	 * @param soundIndex
	 *            it's not the index as used in {@link #playSound(int)}, but the
	 *            position in the category, e.g. 5 = 5th sound in the category.
	 * @param renderLine
	 *            set to true to play on speakers so you hear it.
	 * @param captureLine
	 *            set to true to play on microphone so others hear it.
	 * @return true on success
	 */
	public boolean playSoundFromCategory(final int categoryIndex, final int soundIndex, final boolean renderLine,
			final boolean captureLine) {
		return isSuccessPrintResponse(sendRequestNoException("DoPlaySoundFromCategory(" + categoryIndex + ", "
				+ soundIndex + ", " + renderLine + ", " + captureLine + ")"));
	}
}