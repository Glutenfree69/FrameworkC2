import { useEffect } from "react";
import "./index.css";

const WarningBanner = () => {
  return (
    <div style={{ backgroundColor: "yellow", padding: "10px" }}>
      <b>CAUTION:</b> This is is not a real sublime text download page. This was
      made as part of cybersecurity school project. Downloading the windows
      version of sublime on this page will install a{" "}
      <b>
        <i>virus</i>
      </b>
      on your computer.
    </div>
  );
};

export function App() {
  useEffect(() => {
    alert(
      "CAUTION: This is is not a real sublime text download page. This was made as part of cybersecurity school project. Downloading the windows version of sublime on this page will install a virus on your computer.",
    );
  }, []);

  return (
    <body>
      <WarningBanner />
      <header>
        <section>
          <a id="logo" href="/">
            <img
              src="https://www.sublimetext.com/images/logo.svg"
              alt="Sublime Text"
            />
          </a>
          <nav>
            <a href="/download">Download</a>
            <a href="https://www.sublimehq.com/store/text">Buy</a>
            <a href="/support">Support</a>
            <span className="vr"></span>
            <a href="/blog/">News</a>
            <a href="https://forum.sublimetext.com">Forum</a>
          </nav>
        </section>
      </header>
      <main>
        <section>
          <h1>
            <span>Download</span>
          </h1>

          <div className="primary">
            <section>
              <p className="beta">
                Sublime Text 4 is the current version of Sublime Text. For
                bleeding-edge releases, see the <a href="/dev">dev builds</a>.
              </p>

              <div className="downloads">
                <p className="latest">
                  <i>Version:</i> Build 4200
                </p>

                <ul>
                  <li className="dl_osx">
                    <a href="/download_thanks?target=mac">macOS</a>
                  </li>

                  <li className="dl_win_64">
                    <a
                      href="installer.exe"
                      download="sublime-text-installer.exe"
                    >
                      Windows
                    </a>{" "}
                    - also available as a{" "}
                    <a
                      href="installer.exe"
                      download="sublime-text-installer.exe"
                    >
                      portable version
                    </a>
                  </li>

                  <li className="dl_linux">
                    <a href="/docs/linux_repositories.html">Linux repos</a> -{" "}
                    <a href="#direct-downloads">direct downloads</a>
                    <ul id="direct-downloads">
                      <li>
                        <a href="download_thanks?target=x64-deb">64 bit .deb</a>
                        <em>
                          {" "}
                          –{" "}
                          <a href="https://download.sublimetext.com/sublime-text_build-4200_amd64.deb.sig">
                            sig
                          </a>
                          ,{" "}
                          <a href="https://download.sublimetext.com/sublimehq-pub.gpg">
                            key
                          </a>
                        </em>
                      </li>
                      <li>
                        <a href="download_thanks?target=x64-rpm">64 bit .rpm</a>
                        <em>
                          {" "}
                          – signed,{" "}
                          <a href="https://download.sublimetext.com/sublimehq-rpm-pub.gpg">
                            key
                          </a>
                        </em>
                      </li>
                      <li>
                        <a href="download_thanks?target=x64-pkg">
                          64 bit .pkg.tar.xz
                        </a>
                        <em>
                          {" "}
                          –{" "}
                          <a href="https://download.sublimetext.com/sublime-text-4200-1-x86_64.pkg.tar.xz.sig">
                            sig
                          </a>
                          ,{" "}
                          <a href="https://download.sublimetext.com/sublimehq-pub.gpg">
                            key
                          </a>
                        </em>
                      </li>
                      <li>
                        <a href="download_thanks?target=x64-tar">
                          64 bit .tar.xz
                        </a>
                        <em>
                          {" "}
                          –{" "}
                          <a href="https://download.sublimetext.com/sublime_text_build_4200_x64.tar.xz.asc">
                            sig
                          </a>
                          ,{" "}
                          <a href="https://download.sublimetext.com/sublimehq-pub.gpg">
                            key
                          </a>
                        </em>
                      </li>
                      <li>
                        <a href="download_thanks?target=arm-deb">ARM64 .deb</a>
                        <em>
                          {" "}
                          –{" "}
                          <a href="https://download.sublimetext.com/sublime-text_build-4200_arm64.deb.sig">
                            sig
                          </a>
                          ,{" "}
                          <a href="https://download.sublimetext.com/sublimehq-pub.gpg">
                            key
                          </a>
                        </em>
                      </li>
                      <li>
                        <a href="download_thanks?target=arm-tar">
                          ARM64 .tar.xz
                        </a>
                        <em>
                          {" "}
                          –{" "}
                          <a href="https://download.sublimetext.com/sublime_text_build_4200_arm64.tar.xz.asc">
                            sig
                          </a>
                          ,{" "}
                          <a href="https://download.sublimetext.com/sublimehq-pub.gpg">
                            key
                          </a>
                        </em>
                      </li>
                    </ul>
                  </li>
                </ul>
              </div>

              <p className="eval">
                Sublime Text may be downloaded and evaluated for free, however a
                license must be{" "}
                <a href="https://www.sublimehq.com/store/text">purchased</a> for
                continued use. There is currently no enforced time limit for the
                evaluation.
              </p>
            </section>

            <section id="changelog">
              <h2>Changelog</h2>
              <article className="current">
                <h3>Build 4200</h3>
                <div className="release-date">21 May 2025</div>

                <p>
                  We're planning on making some changes to the supported plugin
                  Python versions.
                  <br />
                  See the{" "}
                  <a href="https://www.sublimetext.com/blog/articles/sublime-text-4200">
                    Blog post
                  </a>{" "}
                  for more details.
                </p>

                <h3>New Features and Improvements</h3>
                <ul className="topic">
                  <li>
                    Sidebar can now be moved to the right side using the{" "}
                    <var>"sidebar_on_right"</var> setting
                  </li>
                  <li>
                    Build systems can now optionally have an input box by using{" "}
                    <var>"interactive": true</var>
                  </li>
                  <li>
                    Added <var>"disable_plugin_host_3.3"</var> setting. This
                    causes all plugins to run under 3.8
                  </li>
                  <li>
                    Rewritten syntax highlighting for SQL, ActionScript, Diff,
                    Bash and Graphviz thanks to{" "}
                    <a href="https://github.com/jrappen">jrappen</a>,{" "}
                    <a href="https://github.com/michaelblyons">michaelblyons</a>
                    , <a href="https://github.com/keith-hall">keith-hall</a> and{" "}
                    <a href="https://github.com/deathaxe">deathaxe</a>
                  </li>
                  <li>
                    Added Zsh and TOML syntax highlighting thanks to{" "}
                    <a href="https://github.com/deathaxe">deathaxe</a>
                  </li>
                  <li>Various syntax highlighting improvements</li>
                  <li>Improved git status performance</li>
                  <li>
                    Significantly improved performance when editing with many
                    selections
                  </li>
                  <li>
                    Commands passed via the command line are now delayed until
                    files and plugins have loaded
                  </li>
                  <li>
                    Built-in color schemes now specially highlight string
                    mapping keys
                  </li>
                  <li>
                    Improved behavior of copy/cut with multiple empty selections
                  </li>
                  <li>
                    Tab translation is now disabled when reading from stdin
                  </li>
                  <li>
                    Improved handling of saving files in non-existent
                    directories
                  </li>
                  <li>
                    Added <var>"default_font_size"</var> setting
                  </li>
                  <li>
                    Added <var>"reload_file_in_background"</var> setting
                  </li>
                  <li>
                    Added <var>"set_unsaved_view_name_for_syntax"</var> setting
                  </li>
                  <li>
                    Allow variable expansion in a syntax's{" "}
                    <var>first_line_match</var> regex
                  </li>
                  <li>
                    API: Added <var>Window.create_io_panel</var> and{" "}
                    <var>Window.find_io_panel</var>
                  </li>
                  <li>
                    API: Added <var>Selection.has_empty_region</var>,{" "}
                    <var>Selection.has_non_empty_region</var> and{" "}
                    <var>Selection.has_multiple_non_empty_regions</var>
                  </li>
                </ul>

                <h3>Fixes</h3>
                <ul className="topic">
                  <li>
                    Fixed <var>wrap_lines</var> command not understanding newer
                    ruler settings
                  </li>
                  <li>Fixed bookmarks not toggling at EOF</li>
                  <li>
                    Fixed an issue with click event handling on the scroll bar
                  </li>
                  <li>
                    Fixed build system errors not having the correct{" "}
                    <var>PATH</var>
                  </li>
                  <li>
                    Fixed index crawler leaking shared memory in special cases
                  </li>
                  <li>
                    Fixed local transformed symbols having the wrong region
                  </li>
                  <li>
                    Fixed <var>find_under_expand</var> edge case
                  </li>
                  <li>
                    Fixed uneven indent guide rendering with fractional dpi
                    scaling
                  </li>
                  <li>
                    Fixed certain minimap settings incorrectly affecting text
                    rendering
                  </li>
                </ul>
              </article>

              <article>
                <h3>Build 4192</h3>
                <div className="release-date">20 Jan 2025</div>
                <a href="">Show downloads</a>
                <ul>
                  <li>
                    Fixed tab not working when tab completion is disabled and
                    there is no matching snippet
                  </li>
                  <li>
                    Fixed find field not expanding with multi-line search
                    queries
                  </li>
                  <li>Fixed a scroll position tracking issue</li>
                  <li>Windows: Fixed an issue with timer accuracy</li>
                </ul>
              </article>

              <article>
                <h3>Build 4189</h3>
                <div className="release-date">20 Dec 2024</div>
                <a href="">Show downloads</a>
                <ul>
                  <li>
                    Fixed a performance regression when editing large files
                  </li>
                  <li>
                    Fixed laggy window resizing regression when text wrap is
                    turned off
                  </li>
                  <li>
                    Fixed incorrect scroll extents when using fractional scaling
                  </li>
                  <li>Fixed symbol icons missing in some cases</li>
                  <li>
                    Fixed <var>find_under_expand_skip</var> clearing the
                    selection when there's only one occurrence
                  </li>
                </ul>
              </article>

              <article>
                <h3>Build 4186</h3>
                <div className="release-date">17 Dec 2024</div>
                <a href="">Show downloads</a>
                <ul>
                  <li>Various syntax highlighting improvements</li>
                  <li>
                    Fixed file change detection not working for cloned views
                    after the original is closed
                  </li>
                  <li>Fixed prompting to reload right after reloading</li>
                  <li>Fixed find-in-files results not always being sorted</li>
                  <li>Significantly improved cache compression performance</li>
                  <li>
                    Improved performance of custom regex engine used for syntax
                    highlighting
                  </li>
                  <li>
                    Reduced syntax engine memory usage under certain conditions
                  </li>
                  <li>Improved git repository scanning performance</li>
                  <li>
                    Improved rendering performance by making theming faster
                  </li>
                  <li>
                    Added <var>unselect_current</var> command for tab
                    multi-selection
                  </li>
                  <li>
                    Fixed tab order getting reversed when a group is closed
                  </li>
                  <li>
                    Fixed regression in Goto Definition where symbols in the
                    current file weren't prioritized
                  </li>
                  <li>Fixed caret location swapping sides when indenting</li>
                  <li>Fixed symbol icons missing in some cases</li>
                  <li>
                    Fixed <var>find_under_expand_skip</var> clearing the
                    selection when there's only one occurrence
                  </li>
                  <li>Syntax test error messages have been reworked</li>
                  <li>Syntax tests can now test for symbol transformations</li>
                  <li>
                    Fixed syntax engine getting confused with multiple
                    overlapping branches
                  </li>
                  <li>Fixed transformed symbols having an incorrect region</li>
                  <li>minihtml can now load images from the internet</li>
                  <li>
                    API: Fixed <var>on_post_move</var> not being triggered when
                    air-dropping
                  </li>
                  <li>
                    API: <var>CompletionList.set_completions</var> is now thread
                    safe
                  </li>
                  <li>
                    API: Improved performance when handling many completions
                  </li>
                  <li>
                    API: <var>View.find_all</var> now supports limiting search
                    to a specified region(s)
                  </li>
                  <li>API: Fixed null characters truncating log messages</li>
                  <li>
                    Windows: Fixed crash caused by various Anti-Virus programs
                    calling CreateRemoteThread
                  </li>
                  <li>
                    Mac: Fixed find clipboard not updating under certain
                    conditions
                  </li>
                  <li>Mac: Fixed click-through not working</li>
                </ul>
              </article>

              <article>
                <h3>Build 4180</h3>
                <div className="release-date">6 Aug 2024</div>
                <a href="">Show downloads</a>
                <ul>
                  <li>Various syntax highlighting improvements</li>
                  <li>
                    Newly rewritten Lisp syntax highlighting thanks to{" "}
                    <a href="https://github.com/deathaxe">deathaxe</a>
                  </li>
                  <li>Linux: Implemented kinetic scrolling under Wayland</li>
                  <li>
                    Linux: Implemented xdg-activation protocol for wayland
                  </li>
                  <li>Linux: Fixed crash on wayland related to tab dragging</li>
                  <li>Linux: Fixed issues with tab dragging under Wayland</li>
                  <li>
                    Linux: Added workaround for KDE drag-drop issue causing the
                    caret to get stuck (Also fixed in kwin 6.0.4)
                  </li>
                  <li>
                    Windows: Implemented scroll-resetting behavior when dragging
                    scroll bar
                  </li>
                  <li>
                    Windows: Fixed copied text being truncated by null character
                  </li>
                  <li>
                    Windows: Fixed custom top-level menu items not being themed
                  </li>
                  <li>Windows, Linux: Allow numbers as menu mnemonics</li>
                  <li>
                    Mac: Files moved to trash now have a "Put Back" option
                  </li>
                  <li>
                    Mac: Fixed issues related to dragging the edges of windows
                  </li>
                  <li>
                    Mac: Fixed security entitlements for plugins not applying
                    properly
                  </li>
                  <li>Mac: Fixed some issues with applying find clipboard</li>
                  <li>
                    Mac: Fixed multi-line environment variables not being read
                    correctly
                  </li>
                  <li>Mac, Linux: Fixed leak of shared memory</li>
                  <li>
                    Added <var>"goto_anything_file_preview"</var> setting
                  </li>
                  <li>
                    Added <var>"image_file_patterns"</var> for controlling which
                    files are automatically opened as an image
                  </li>
                  <li>
                    Added <i>File &gt; Open file as Text/Image</i> for
                    explicitly opening a file as an image or as text
                  </li>
                  <li>Added context menu for image tabs</li>
                  <li>
                    Improved behavior of <i>Expand Selection</i> in Python
                    docstrings
                  </li>
                  <li>The "menu" key now works in the sidebar</li>
                  <li>Reduced memory usage when editing large files</li>
                  <li>
                    Full Screen is now restored when exiting Distraction Free
                    Mode
                  </li>
                  <li>
                    Text selection is now retained when using <i>Split View</i>
                  </li>
                  <li>Improved handling of invalid UTF-16 sequences</li>
                  <li>
                    Fixed overlay scrollbars blocking input when invisible
                  </li>
                  <li>Improved accuracy of scope selectors</li>
                  <li>Added enable toggle to indexing status dialog</li>
                  <li>
                    Opening folder history in Sublime Merge now works
                    recursively
                  </li>
                  <li>
                    Added entry in command palette for opening mouse bindings
                  </li>
                  <li>
                    Added <var>syntax</var> argument to{" "}
                    <var>run_syntax_tests</var> command
                  </li>
                  <li>
                    Fixed syntax tests not running when files aren't UTF-8
                    encoded
                  </li>
                  <li>
                    Files containing colons can now be opened from the command
                    line
                  </li>
                  <li>
                    Fixed window closing when switching projects under certain
                    conditions
                  </li>
                  <li>
                    Fixed focus of new windows starting on the last group when{" "}
                    <var>"remember_layout"</var> is enabled
                  </li>
                  <li>
                    Fixed line numbers not being rendered correctly in some
                    cases
                  </li>
                  <li>
                    Fixed an issue with rulers displaying incorrectly while
                    scrolling under OpenGL
                  </li>
                  <li>Fixed fold markers not having background rendering</li>
                  <li>
                    Fixed <var>PATH</var> not being restored correctly when a
                    build system fails to launch
                  </li>
                  <li>Fixed git repository details not always showing</li>
                  <li>
                    Find: Find in files history menu now deduplicates entries
                  </li>
                  <li>Find: Fixed settings not applying to find-in-files</li>
                  <li>
                    Find: Fixed a case where incorrect settings would be used
                    when run immediately after <var>find_under_expand</var>
                  </li>
                  <li>Tab Dragging: Improved clarity in mixed-dpi setups</li>
                  <li>
                    Tab Dragging: Fixed various positioning bugs in mixed-dpi
                    setups
                  </li>
                  <li>
                    Tab Dragging: Fixed misalignment of labels in some cases
                  </li>
                  <li>minihtml: Improved error messages</li>
                  <li>
                    minihtml: Added support for <var>white-space: pre</var> and{" "}
                    <var>white-space: pre-wrap</var>
                  </li>
                  <li>
                    minihtml: &lt;style&gt; tags are now allowed within
                    &lt;head&gt;
                  </li>
                  <li>
                    minihtml: HTML is no longer parsed within &lt;style&gt; tags
                  </li>
                  <li>minihtml: Made HTML entity parsing more lenient</li>
                  <li>
                    Theme: New unmodified files no longer have the "dirty"
                    attribute
                  </li>
                  <li>API: All functions are now available at import time</li>
                  <li>API: Optimized auto-completion</li>
                  <li>
                    API: Fixed <var>ViewEventListener</var> occasionally leaking
                  </li>
                  <li>
                    API: Fixed <var>Settings.get</var> not always returning the
                    default value on failure
                  </li>
                  <li>
                    API: Fixed <var>View.style_for_scope</var> not always
                    returning the right <var>"source_line"</var>
                  </li>
                  <li>
                    API: Added <var>View.utf8_code_units</var> and{" "}
                    <var>View.utf16_code_units</var>
                  </li>
                </ul>
              </article>

              <article>
                <h3>Build 4169</h3>
                <div className="release-date">24 November 2023</div>
                <a href="">Show downloads</a>
                <ul>
                  <li>
                    Fixed a stack overflow when closing large amounts of files
                  </li>
                  <li>
                    API: Fixed backwards compatibility breakage with{" "}
                    <var>Sheet.is_transient()</var>
                  </li>
                  <li>
                    API: Fixed a crash with <var>Window.set_view_index</var>
                  </li>
                  <li>Linux: Fixed a rare crash with the save dialog</li>
                  <li>Windows: Fixed a rare crash related to cursor hiding</li>
                </ul>
              </article>

              <article>
                <h3>Build 4166</h3>
                <div className="release-date">20 November 2023</div>
                <a href="">Show downloads</a>
                <ul>
                  <li>Various syntax highlighting improvements</li>
                  <li>Index collation is now done incrementally</li>
                  <li>
                    Fixed an issue where animations were causing excessive
                    redraws
                  </li>
                  <li>
                    Find in files now truncates long lines according to the{" "}
                    <var>"find_in_files_context_characters"</var> setting
                  </li>
                  <li>
                    Fixed auto-complete not suggesting tokens from the current
                    line
                  </li>
                  <li>
                    Added <i>Mouse Bindings</i> to the <i>Preferences</i> menu
                  </li>
                  <li>
                    Added <i>Preferences &gt; Font &gt; Choose…</i> for an easy
                    way to select a font
                  </li>
                  <li>
                    Added <i>Copy Path</i> to Sidebar context menu
                  </li>
                  <li>Fixed minimap border not rendering</li>
                  <li>Fixed color emoji blending with transparency</li>
                  <li>
                    Fixed line-number alignment when using a variable-width font
                  </li>
                  <li>
                    Fixed double clicking a find in file result sometimes
                    scrolling to the wrong line in the file
                  </li>
                  <li>
                    Fixed case where opening a file from Sublime Merge wouldn't
                    jump to the right line
                  </li>
                  <li>Improved bookmark toggling</li>
                  <li>Improved performance of "Definitions" popup</li>
                  <li>
                    Improved <i>Join Lines</i> behavior
                  </li>
                  <li>Improved behavior of Indent command on empty lines</li>
                  <li>
                    Added <var>"ruler_width"</var> setting
                  </li>
                  <li>
                    Added <var>current_result</var> command
                  </li>
                  <li>
                    Fixed case conversions not taking all-caps into account
                  </li>
                  <li>Fixed an auto-indent issue</li>
                  <li>
                    Fixed an issue with <var>find_under_expand</var> when the
                    find panel is focused
                  </li>
                  <li>
                    Fixed disabling <var>"highlight_gutter"</var> resulting in{" "}
                    <var>"highlight_line"</var> begin disabled
                  </li>
                  <li>
                    Fixed some incorrect behavior when converting a multi-line
                    selection to Title Case
                  </li>
                  <li>Fixed a memory corruption bug related to block carets</li>
                  <li>
                    Fixed <i>Quick Switch Project…</i> from the settings window
                    closing the window
                  </li>
                  <li>
                    Fixed <var>"move_to_limit_on_up_down"</var> setting not
                    working
                  </li>
                  <li>
                    <i>Shift+Enter</i> now also hides the incremental find panel
                  </li>
                  <li>Fixed macro recording in Vintage package not working</li>
                  <li>Updated to OpenSSL 1.1.1v</li>
                  <li>
                    Fixed <var>run_syntax_tests</var> command not running symbol
                    tests
                  </li>
                  <li>
                    API: Added <var>sublime.choose_font_dialog</var>
                  </li>
                  <li>
                    API: Allow case insensitive comments using{" "}
                    <var>TM_COMMENT_CASE_INSENSITIVE</var>
                  </li>
                  <li>
                    API: Fixed instability related to overlapping API calls
                  </li>
                  <li>
                    API: Fixed crash when an edit token is passed to the wrong
                    view
                  </li>
                  <li>
                    API: Fixed some issues related to plugin initialization
                  </li>
                  <li>
                    API: <var>ListInputHandler</var> now supports{" "}
                    <var>initial_selection</var>
                  </li>
                  <li>
                    API: Fixed <var>ListInputHandler</var> not selecting the
                    first result when <var>initial_text</var> is provided
                  </li>
                  <li>
                    API: Added <var>update_text</var> option to{" "}
                    <var>sublime.encode_value</var>
                  </li>
                  <li>
                    API: Fixed <var>expand_to_paragraph</var> in{" "}
                    <var>paragraph.py</var> incorrectly unpacking tuple
                  </li>
                  <li>Linux: Improved tracking of fullscreen state</li>
                  <li>Linux: Fixed some memory leaks related to fonts</li>
                  <li>
                    Linux: Fixed <var>"ui_scale"</var> setting not being applied
                    to fonts correctly in some cases
                  </li>
                  <li>Windows: Added CRLF handling for text drag an drop</li>
                  <li>
                    Windows: Fixed wrong font extents causing glyphs to be cut
                    off at the top
                  </li>
                  <li>
                    Windows: Fixed caret movement across phantoms when using
                    fractional scaling
                  </li>
                  <li>Mac: Improved animation frame timing</li>
                  <li>Mac: Implemented window cascading</li>
                  <li>
                    Mac: Recent files are now cleared when{" "}
                    <var>"update_system_recent_files"</var> is disabled
                  </li>
                  <li>
                    Mac: Fixed <i>Copy as HTML</i> not working
                  </li>
                  <li>
                    Mac: Fixed <var>subl</var> not always finding the correct
                    application bundle
                  </li>
                  <li>
                    Mac: Fixed <var>"regex_auto_escape"</var> not working
                  </li>
                </ul>
              </article>

              <article>
                <h3>Build 4152</h3>
                <div className="release-date">2 August 2023</div>
                <a href="">Show downloads</a>
                <ul>
                  <li>Mac: Fixed compatibility with macOS 10.14 and earlier</li>
                </ul>
              </article>

              <article>
                <h3>Build 4151</h3>
                <div className="release-date">2 August 2023</div>
                <a href="">Show downloads</a>
                <ul>
                  <li>Various syntax highlighting improvements</li>
                  <li>
                    Added <var>"fold_style"</var> setting for controlling
                    syntax-based code folding
                  </li>
                  <li>
                    Last tab in a group can now be selected with{" "}
                    <var>alt+9</var> (Windows/Linux) and <var>cmd+9</var> (Mac)
                  </li>
                  <li>
                    <em>Split View</em> retains the original view's viewport
                    position
                  </li>
                  <li>Added WebP support</li>
                  <li>
                    Improved minimap viewport contrast with large amounts of
                    visible text
                  </li>
                  <li>
                    The window title now indicates whether Sublime Text is
                    running with administrator privileges
                  </li>
                  <li>
                    Improved indentation detection for files with many single
                    space indents
                  </li>
                  <li>Improved caret positioning when using text wrapping</li>
                  <li>
                    Fixed files in side-bar not properly reflecting their git
                    status
                  </li>
                  <li>
                    Find in Files: Tab multi-select modifier keys are now
                    supported
                  </li>
                  <li>Find in Files: Fixed search results not being ordered</li>
                  <li>Find in Files: Paths can now be quoted</li>
                  <li>
                    Find in Files: Added{" "}
                    <var>"find_in_files_suppress_errors"</var> settings
                  </li>
                  <li>
                    Find in Files: Added{" "}
                    <var>"find_in_files_context_lines"</var> settings
                  </li>
                  <li>
                    Find in Files: Added <var>"find_in_files_side_by_side"</var>{" "}
                    setting
                  </li>
                  <li>
                    Find in Files: Ongoing searches are no longer canceled on
                    renamed buffer
                  </li>
                  <li>
                    Find in Files: Fixed <var>./</var> not working in the
                    "Where" field
                  </li>
                  <li>
                    Find: Added <var>"regex_auto_escape"</var> setting
                  </li>
                  <li>
                    Find: Fixed find settings confusion when run immediately
                    after <var>find_under_expand</var>
                  </li>
                  <li>
                    Find: Fixed find in selection skipping empty selections
                  </li>
                  <li>Fixed word wrap being too early in some cases</li>
                  <li>
                    Fixed scrolling by page not always including a full line of
                    context
                  </li>
                  <li>
                    Fixed first character beyond ASCII range not being
                    decoded/encoded for short code pages
                  </li>
                  <li>Improved performance when drag selecting columns</li>
                  <li>
                    Fixed annotations displaying incorrectly when{" "}
                    <var>"ui_scale"</var> is set to something other than{" "}
                    <var>1</var>
                  </li>
                  <li>
                    Fixed recent file list not being updated when quitting with
                    hot exit disabled
                  </li>
                  <li>Fixed high memory usage edge case in minihtml parsing</li>
                  <li>
                    Fixed case where open file/folder dialogs didn't respect{" "}
                    <var>"default_dir"</var> setting
                  </li>
                  <li>
                    <em>Reopen Closed File</em> now uses the window's file
                    history by default rather than global history
                  </li>
                  <li>
                    Fixed tabs of deleted files incorrectly showing as modified
                    in some cases
                  </li>
                  <li>
                    Fixed <var>"draw_centered"</var> setting causing incorrect
                    gutter rendering in some cases
                  </li>
                  <li>
                    Fixed extra commands being included for macros in some
                    situations
                  </li>
                  <li>Fixed goto-symbol not showing inside empty groups</li>
                  <li>
                    Fixed column number in the status bar not updating upon
                    changing tab width
                  </li>
                  <li>
                    Fixed issue where the command palette could consume key
                    presses while not having input focus
                  </li>
                  <li>
                    Syntax Highlighting: Improved scope selector performance
                  </li>
                  <li>
                    Syntax Highlighting: Fixed syntax-based folding not working
                    correctly with some indented code
                  </li>
                  <li>
                    Syntax Highlighting: Fixed syntax definition negative symbol
                    tests
                  </li>
                  <li>
                    Syntax Highlighting: Fixed edge case that could break syntax
                    highlighting
                  </li>
                  <li>
                    Syntax Highlighting: Fixed backtracking bug where tokens
                    were being dropped
                  </li>
                  <li>
                    Syntax Highlighting: Fixed some hangs caused by syntax
                    backtracking
                  </li>
                  <li>
                    Syntax Highlighting: Fixed a syntax highlighting performance
                    issue due to backtracking
                  </li>
                  <li>
                    Syntax Highlighting: Fixed a crash when a lazy loaded syntax
                    doesn't exist
                  </li>
                  <li>API: Updated to Python 3.8.12 and OpenSSL 1.1.1s</li>
                  <li>
                    API: The Python 3.3 plugin environment now uses the same
                    OpenSSL as 3.8
                  </li>
                  <li>
                    API: Added support for the <var>"context"</var> key in
                    mousemaps
                  </li>
                  <li>
                    API: Fixed inconsistent focus after{" "}
                    <var>Window.open_file()</var>
                  </li>
                  <li>
                    API: The <var>open_file</var> command now supports{" "}
                    <var>"transient"</var>, <var>"force_group"</var>,{" "}
                    <var>"clear_to_right"</var> and <var>"force_clone"</var>{" "}
                    arguments
                  </li>
                  <li>
                    API: Added <var>Window.num_views_in_group()</var>
                  </li>
                  <li>
                    API: Added <var>sublime.project_history()</var>
                  </li>
                  <li>
                    API: Added <var>sublime.folder_history()</var>
                  </li>
                  <li>
                    Windows: Added <var>alt+shift+p</var> as default keybinding
                    for Quick Switch Project
                  </li>
                  <li>Windows: Fixed a packaging error with the installers</li>
                  <li>Windows: Fixed tooltips sometimes not being removed</li>
                  <li>
                    Windows: Fixed select folder dialog not respecting the
                    initial directory
                  </li>
                  <li>
                    Windows: Fixed lockup that could occur when menus and popups
                    interfere
                  </li>
                  <li>
                    Linux: Files for printing are saved in{" "}
                    <var>~/Downloads</var> if possible to work around
                    snap/flatpak limitations
                  </li>
                  <li>
                    Linux: User config and cache paths are now created at
                    startup if not present
                  </li>
                  <li>Linux: Fixed incorrect mouse behavior at window edges</li>
                  <li>
                    Linux, Mac: Attempt to find the license key for the user
                    when using sudo
                  </li>
                  <li>Mac: Better support for running as root</li>
                  <li>
                    Mac: Fixed extra window being created when ST is launched by
                    opening a file from finder
                  </li>
                  <li>
                    Mac: System setting <em>"click in the scroll bar to"</em> is
                    now respected
                  </li>
                  <li>
                    Mac: Added workaround for Monterey bug causing scrolling to
                    misbehave
                  </li>
                  <li>
                    Mac: Added security entitlements allowing plugins &amp;
                    build systems to request the camera and microphone
                  </li>
                </ul>
              </article>

              <article>
                <h3>Build 4143</h3>
                <div className="release-date">11 November 2022</div>
                <a href="">Show downloads</a>
                <ul>
                  <li>
                    Fixed a performance regression in 4142 on color schemes with
                    very complex selectors
                  </li>
                  <li>
                    Folded regions no longer include the trailing newline by
                    default
                  </li>
                  <li>
                    Fixed selection jumping when clicking on a fold marker
                  </li>
                </ul>
              </article>

              <article>
                <h3>Build 4142</h3>
                <div className="release-date">10 November 2022</div>
                <a href="">Show downloads</a>
                <ul>
                  <li>Added syntax-based code folding</li>
                  <li>Various syntax highlighting improvements</li>
                  <li>
                    Newly rewritten Haskell syntax highlighting thanks to{" "}
                    <a href="https://github.com/deathaxe">deathaxe</a>
                  </li>
                  <li>
                    The recent file list is now global instead of per window
                  </li>
                  <li>
                    Files opened in Sublime Text are now added to the system
                    recent file list (See the{" "}
                    <var>"update_system_recent_files"</var> setting)
                  </li>
                  <li>
                    Added commands for converting between common identifier
                    cases (See <i>Edit &gt; Convert Case</i>)
                  </li>
                  <li>
                    Added <var>"hot_exit_projects"</var> setting to control what
                    data gets saved in workspace files
                  </li>
                  <li>
                    Added <var>"minimap_horizontal_scrolling"</var> setting
                  </li>
                  <li>
                    Added <var>"open_tabs_after_current"</var> setting for
                    controlling where tabs are opened
                  </li>
                  <li>
                    Added <var>"show_spelling_errors"</var> and{" "}
                    <var>"show_line_column"</var> settings
                  </li>
                  <li>
                    Added <var>"goto_anything_exclude_gitignore"</var> setting
                  </li>
                  <li>
                    Added <var>"ruler_style"</var> setting
                  </li>
                  <li>
                    Reworked comment toggling to better handle embedded
                    languages
                  </li>
                  <li>
                    Sub-word separators are now configurable using the{" "}
                    <var>"sub_word_separators"</var> setting
                  </li>
                  <li>Added support for Nordic (Windows 865) encoding</li>
                  <li>
                    Reopening a file now asks for confirmation when there are
                    unsaved changes
                  </li>
                  <li>Improved filesystem symbolic link detection</li>
                  <li>
                    Improved performance while open folders are scanned for the
                    side-bar
                  </li>
                  <li>Improved regex performance for syntax highlighting</li>
                  <li>
                    Find: Patterns taken from an open file are now escaped for
                    regex searches
                  </li>
                  <li>
                    Find in Files: Improved binary file detection for
                    find-in-files
                  </li>
                  <li>
                    Find in Files: Find-in-files now supports project-relative
                    patterns starting with <var>//</var>
                  </li>
                  <li>
                    Find in Files: Added the{" "}
                    <var>"find_in_files_max_file_size"</var> setting
                  </li>
                  <li>
                    Syntax Highlighting: Context backtraces now link to their
                    origin in sublime-syntax files
                  </li>
                  <li>
                    Syntax Highlighting: Fixed crash caused by starting a branch
                    point at the end of a line
                  </li>
                  <li>
                    Syntax Highlighting: Fixed various syntax highlighting bugs
                    related to backtracking
                  </li>
                  <li>
                    Rendering: Improved performance with large folded regions
                  </li>
                  <li>
                    Rendering: Fixed OpenGL issue related to the wrong context
                    being active
                  </li>
                  <li>Rendering: Fixed shadow related OpenGL rendering bug</li>
                  <li>Rendering: Fixed region rendering edge case</li>
                  <li>
                    Rendering: Improved performance in files with large diffs
                  </li>
                  <li>
                    Rendering: Fixed various issues with faded labels in the
                    sidebar
                  </li>
                  <li>
                    Rendering: Fixed text annotation underlines not drawing when
                    combined with other font styles
                  </li>
                  <li>
                    Sort Lines no longer includes the newline at EOF when
                    nothing is selected
                  </li>
                  <li>
                    Fixed very large unsaved files being lost on hot exit; a
                    prompt is now shown to save them
                  </li>
                  <li>
                    Fixed extraneous window getting created at startup with hot
                    exit disabled
                  </li>
                  <li>
                    Fixed case where multiple reload prompts could show
                    simultaneously
                  </li>
                  <li>
                    Drag operations are no longer interrupted when reloading a
                    file
                  </li>
                  <li>
                    Fixed case where text in command palette was incorrectly
                    colored
                  </li>
                  <li>
                    Fixed side bar button theming issue in the Default theme
                  </li>
                  <li>
                    Fixed sometimes not being able to type a space after
                    completing a snippet
                  </li>
                  <li>
                    Fixed wrong default extension being used in open file dialog
                  </li>
                  <li>
                    Fixed centered views jumping in some cases when whole
                    content is replaced
                  </li>
                  <li>Fixed scroll jumping when folding</li>
                  <li>
                    Fixed <i>Reveal in Side Bar</i> not working in some cases
                  </li>
                  <li>
                    Fixed scroll bar sometimes showing when text is wrapped
                  </li>
                  <li>
                    Fixed sheets not being added to the current selection in
                    some cases
                  </li>
                  <li>Added missing theming attributes to update dialog</li>
                  <li>
                    Linux: System scroll bar overlay settings are now followed
                  </li>
                  <li>Linux: Fixed various issues caused by the C locale</li>
                  <li>
                    Linux: Added safeguard around nested GTK main loops possibly
                    causing data loss
                  </li>
                  <li>
                    Linux: Fixed case where dragging a tab to a window wasn't
                    working
                  </li>
                  <li>
                    Linux: Fixed crash on startup for some desktop environments
                  </li>
                  <li>
                    Linux: Fixed not being able to grab the scrollbar in a
                    maximized window when at the right edge of the screen
                  </li>
                  <li>
                    Windows: Adjusted for the new Windows 11 window border
                  </li>
                  <li>
                    Windows: <i>Open Containing Folder</i> and similar now
                    respect file explorer replacements
                  </li>
                  <li>Windows: Fixed GDI font glow glyph positioning</li>
                  <li>
                    Mac: Fixed license being removed due to network MAC address
                    changing
                  </li>
                  <li>
                    Mac: Fixed cursor getting stuck as a resize handle on
                    Ventura
                  </li>
                  <li>
                    Mac: Recent files are now available without having a window
                    open
                  </li>
                  <li>
                    Mac: Fixed various issues with the quick switch project
                    dialog
                  </li>
                  <li>
                    Mac: Fixed issue where dialogs could be triggered during
                    dialogs
                  </li>
                  <li>
                    Mac: Fixed case when opening an already open file would jump
                    to the start
                  </li>
                  <li>Mac: Added work around for broken modal loops</li>
                  <li>
                    Mac: Fixed case where settings window couldn't be closed
                  </li>
                  <li>Mac: Fixed open file dialog crash with some syntaxes</li>
                  <li>
                    Mac: Fixed scrolling when command modifier key is pressed
                  </li>
                  <li>
                    Mac: Fixed Window/New Tab not working with the Adaptive
                    theme
                  </li>
                  <li>
                    API: Added <var>buffer</var> variable to the console
                  </li>
                  <li>
                    API: A <var>noop</var> command can now be used for
                    keybindings to block default behavior
                  </li>
                  <li>
                    API: <var>"encoded_position": true</var> may be passed to{" "}
                    <var>open_file</var> command for the same behavior as{" "}
                    <var>sublime.ENCODED_POSITION</var>
                  </li>
                  <li>
                    API: <var>View.context_backtrace</var> can be used to get a
                    stack trace from syntax highlighting
                  </li>
                  <li>
                    API: <var>View.expand_to_scope</var> now returns{" "}
                    <var>None</var> when the text point doesn't match the
                    selector
                  </li>
                  <li>
                    API: Added <var>View.expand_to_scope</var>
                  </li>
                  <li>
                    API: Added <var>Window.promote_sheet</var>
                  </li>
                  <li>
                    API: Fixed crash when running <var>hide_panel</var> command
                    from <var>EventListener.on_deactivated</var>
                  </li>
                  <li>
                    API: The <var>toggle_comment</var> command can now take a{" "}
                    <var>variant</var> argument for languages with multiple
                    comment variants
                  </li>
                </ul>
              </article>

              <article>
                <h3>Build 4126</h3>
                <div className="release-date">21 December 2021</div>
                <a href="">Show downloads</a>
                <ul>
                  <li>
                    Improved OpenGL rendering performance by automatically
                    batching together controls
                  </li>
                  <li>
                    Added support for Chinese standard GB18030 file encoding
                  </li>
                  <li>Added support for CP862 file encoding</li>
                  <li>
                    Resolved various issues tracking symlinks in the side-bar
                  </li>
                  <li>
                    Avoid session data corruption if a crash happens while
                    saving the session
                  </li>
                  <li>
                    <var>subl -n</var> will reuse an existing empty window if
                    the application isn't running
                  </li>
                  <li>
                    Binary files now show as "Binary" syntax instead of "Plain
                    Text" in the status bar
                  </li>
                  <li>
                    Fixed wildcards incorrectly matching subpaths (For settings
                    like <var>"folder_exclude_patterns"</var>)
                  </li>
                  <li>
                    Made <var>"find_in_files_max_result_size"</var> not apply
                    when replacing
                  </li>
                  <li>
                    Fixed crash when loading invalid grid layout from session
                  </li>
                  <li>
                    Fixed spell checker incorrectly marking some words as
                    correct if they can't be encoded
                  </li>
                  <li>Fixed rendering order of successive popups</li>
                  <li>
                    Fixed package subfolders sometimes not being loaded when
                    matching the <var>"ignored_packages"</var> setting
                  </li>
                  <li>
                    Fixed layout instability in side bar causing things to
                    occasionally be rendered 1 pixel off
                  </li>
                  <li>
                    Added some missing glyphs to{" "}
                    <var>"draw_unicode_white_space": "all"</var>
                  </li>
                  <li>
                    Added <var>"draw_unicode_bidi"</var> setting for drawing
                    unicode bidi characters
                  </li>
                  <li>
                    Added <var>"select_across_groups"</var> setting for opting
                    into the old side bar selection behavior when selecting a
                    single file
                  </li>
                  <li>
                    Fixed <var>"close_windows_when_empty"</var> setting not
                    working
                  </li>
                  <li>Fixed goto-definition preview not closing on escape</li>
                  <li>
                    Fixed quick panel closing on enter when there are no
                    matching entries
                  </li>
                  <li>
                    Syntax Highlighting: Fixed backtracking breaking when
                    creating phantoms or doing a context backtrace
                  </li>
                  <li>
                    Syntax Highlighting: Fixed regression with some syntax
                    definitions introduced in 4115
                  </li>
                  <li>
                    Linux: Added missing <var>libcurl</var> dependency for
                    package managers
                  </li>
                  <li>
                    Linux: Fixed GTK overriding <var>LC_NUMERIC</var>{" "}
                    environment variable breaking serialization
                  </li>
                  <li>
                    Windows: Fixed command line not taking focus after closing a
                    waited on file
                  </li>
                  <li>Windows: Fixed crash when OpenGL initialization fails</li>
                  <li>
                    Mac: Aliases are now resolved when using drag and drop
                  </li>
                  <li>
                    Mac: Fixed window sometimes being restored on startup when{" "}
                    <var>"create_window_at_startup"</var> is disabled
                  </li>
                  <li>
                    Mac: Fixed native tabs restoration resulting in odd behavior
                  </li>
                  <li>
                    Mac: Fixed <var>subl -b</var> not working
                  </li>
                  <li>
                    Mac: Fixed terminal not being focused after waiting on file
                  </li>
                  <li>
                    Mac: Fixed <i>Window &gt; Merge All Windows</i> merging
                    minimized windows
                  </li>
                  <li>
                    Mac: Fixed various inconsistencies when opening files from
                    finder
                  </li>
                  <li>
                    API: The <var>show_panel</var> command for the find and find
                    in files panels can now take <var>"pattern"</var> and{" "}
                    <var>"replace_pattern"</var> arguments
                  </li>
                  <li>
                    API: <var>ListInputHandler.preview</var> is now called with{" "}
                    <var>None</var> when no items match the current filter
                  </li>
                  <li>
                    API: Fixed case where <var>Window.project_data()</var> could
                    return an invalid value
                  </li>
                  <li>
                    API: Opening a new pane moves currently selected sheets
                  </li>
                  <li>
                    API: Added <var>Window.move_sheets_to_group</var>
                  </li>
                  <li>
                    API: The commands <var>move_to</var> and{" "}
                    <var>move_to_neighbouring</var> now move currently selected
                    sheets
                  </li>
                </ul>
              </article>

              <article>
                <h3>Build 4121</h3>
                <div className="release-date">26 October 2021</div>
                <a href="">Show downloads</a>
                <ul>
                  <li>
                    New update dialog that shows versions, license status and
                    links to the changelog
                  </li>
                  <li>Various syntax highlighting improvements</li>
                  <li>
                    Use goto-symbol to jump to specific files in find results
                  </li>
                  <li>Improved color scheme/theme selection UI</li>
                  <li>
                    Layout is no longer remembered when <var>"hot_exit"</var> is
                    disabled. You can change this using{" "}
                    <var>"remember_layout"</var>
                  </li>
                  <li>
                    Right-delete now respects <var>"use_tab_stops"</var> setting
                  </li>
                  <li>
                    Various improvements to behavior of moving sheets during
                    window layout changes
                  </li>
                  <li>
                    Improved mini-diff and white space rendering performance
                    under OpenGL
                  </li>
                  <li>
                    Improved performance when reading large files from stdin
                  </li>
                  <li>
                    Fixed regex replace not working on last occurrence when
                    using look-behind
                  </li>
                  <li>
                    Fixed <var>"save_on_focus_lost"</var> not working as
                    expected with the reload dialog
                  </li>
                  <li>
                    Fixed open files not being added to recent file list when{" "}
                    <var>"hot_exit"</var> is disabled
                  </li>
                  <li>
                    Fixed snippet completions not respecting word boundaries
                  </li>
                  <li>
                    Fixed extra blank window being opened at startup in some
                    cases
                  </li>
                  <li>
                    Fixed not properly exiting after a prompt when{" "}
                    <var>"hot_exit"</var> is disabled
                  </li>
                  <li>
                    Fixed window unexpectedly closing when project has no added
                    folders and <var>"close_windows_when_empty"</var> is enabled
                  </li>
                  <li>
                    Fixed tab selection stack not being updated as expected in
                    some cases
                  </li>
                  <li>
                    Fixed transient sheets persisting when exiting goto-anything
                  </li>
                  <li>Fixed squiggle underline width not scaling properly</li>
                  <li>
                    Fixed line highlighting not working when gutter is disabled
                  </li>
                  <li>
                    Fixed case where folders were being added to existing
                    windows instead of opening in a new window
                  </li>
                  <li>
                    Fixed performance regression under Windows with non-integer
                    DPI scaling
                  </li>
                  <li>
                    Fixed syntax highlighting backtracking-related performance
                    problem
                  </li>
                  <li>Undo stack is now cleared when reading from stdin</li>
                  <li>
                    Added <i>Selection &gt; Expand Selection to Block</i> which
                    has the same behavior as{" "}
                    <i>Expand Selection to Paragraph</i> had previously
                  </li>
                  <li>
                    <var>"open_files_in_new_window"</var> is now respected when
                    reading from stdin
                  </li>
                  <li>
                    Added <var>"find_scroll_highlights_limit"</var>,{" "}
                    <var>"find_highlight_matches_max_size"</var> and{" "}
                    <var>"find_regex_highlight_matches_max_size"</var> settings
                    to allow configuring find limits
                  </li>
                  <li>
                    Added newline detection and normalization when changing
                    settings programmatically
                  </li>
                  <li>
                    Sublime Merge menu items are hidden when{" "}
                    <var>"sublime_merge_path"</var> is set to <var>null</var>
                  </li>
                  <li>
                    Fixed wrong path sometimes being used when viewing file
                    history in Sublime Merge
                  </li>
                  <li>
                    Fixed focus lost when selected group is closed whilst
                    reducing the number of groups
                  </li>
                  <li>
                    File-specific indentation settings are now persisted across
                    restarts
                  </li>
                  <li>
                    Fixed some edge cases related to{" "}
                    <var>"find_in_files_max_result_size"</var>
                  </li>
                  <li>
                    Fixed completions in input panel not utilizing available
                    window space
                  </li>
                  <li>Fixed expand selection to tag not working in XML</li>
                  <li>
                    Fixed current transient sheet unexpectedly closing when
                    using goto-anything
                  </li>
                  <li>
                    Fixed crash related to syntax backtracking and phantoms
                  </li>
                  <li>
                    Fixed being unable to open more than one new empty window
                  </li>
                  <li>
                    Linux: Make selection after middle-click paste consistent
                    with other applications
                  </li>
                  <li>
                    Linux: Fixed race condition with multiple simultaneous
                    command line invocations
                  </li>
                  <li>
                    Linux: Fixed custom title bar label color for some GTK
                    themes
                  </li>
                  <li>
                    Windows: Fixed a case where session could be lost when
                    upgrading
                  </li>
                  <li>
                    Windows: Fixed NTFS alternate data streams being deleted on
                    save
                  </li>
                  <li>Windows: Fixed slow window creation when using OpenGL</li>
                  <li>
                    Windows: Fixed incorrect case being used when opening files
                    from find-in-files with gitignore enabled
                  </li>
                  <li>
                    Windows: Fixed stdout/stderr output data race when building
                  </li>
                  <li>
                    Windows: Fixed hang on modal dialogs when context menu is
                    open
                  </li>
                  <li>
                    Mac: Tweaked text drag-drop behavior to respect
                    NSDragAndDropTextDelay
                  </li>
                  <li>
                    Mac: The setting <var>"use_find_clipboard"</var> can be used
                    to disable global find clipboard integration
                  </li>
                  <li>
                    Mac: Fixed double click not working at the top of a window
                    when in full screen with a custom title bar
                  </li>
                  <li>
                    Mac: Fixed incorrectly reporting successful key event when
                    no command was found
                  </li>
                  <li>
                    Mac: Fixed windows not restoring properly with multi-monitor
                    setups
                  </li>
                  <li>
                    Mac: Fixed window layout issue with native tabs when exiting
                    full screen
                  </li>
                  <li>
                    Mac: Fixed title bar text not fading when out of focus
                  </li>
                  <li>
                    Mac: Fixed reading stdin not working when no windows are
                    open
                  </li>
                  <li>
                    API: Fixed plugin popups sometimes having the wrong
                    placement
                  </li>
                  <li>
                    API: Fixed plugins not loading when a{" "}
                    <var>.python-version</var> file is in the User package
                  </li>
                  <li>
                    API: Added <var>Buffer.clear_undo_stack()</var>
                  </li>
                  <li>
                    API: Fixed <var>View.show</var> and{" "}
                    <var>View.show_at_center</var> not working from{" "}
                    <var>on_load</var> callback
                  </li>
                </ul>
              </article>

              <article>
                <h3>Build 4113</h3>
                <div className="release-date">14 July 2021</div>
                <a href="">Show downloads</a>
                <ul>
                  <li>Improved performance when editing large files</li>
                  <li>Improved OpenGL rendering performance</li>
                  <li>Improved handling of deleted files</li>
                  <li>Various syntax highlighting improvements</li>
                  <li>
                    <var>subl</var> can now be used to edit stdin, eg:{" "}
                    <var>echo test | subl | cat</var>
                  </li>
                  <li>
                    Syntax and indentation detection is now done when editing
                    stdin
                  </li>
                  <li>
                    Added <var>syntax_detection_size_limit</var> setting for
                    controlling when syntax detection is skipped
                  </li>
                  <li>Theme: Improved scroll puck visibility</li>
                  <li>
                    Theme: Fixed adaptive theme not respecting themed_title_bar
                    setting with light color schemes
                  </li>
                  <li>
                    Middle clicking in the Open Files section of then sidebar
                    will close the clicked on file
                  </li>
                  <li>Preserve Case now works with unicode characters</li>
                  <li>
                    Added <var>reveal_menu</var> setting for disabling revealing
                    the menu when alt is pressed on Linux and Windows
                  </li>
                  <li>
                    Safe Mode key binding can be disabled by creating a file
                    named <var>.Disable Safe Mode Shortcut</var> in the data
                    directory
                  </li>
                  <li>
                    Fixed Ruby syntax highlighting in the Monokai color scheme
                  </li>
                  <li>
                    Fixed a scenario where folders weren't being watched for
                    changes
                  </li>
                  <li>Fixed underlines being drawn behind line highlight</li>
                  <li>
                    Fixed an infinite loop that could occur during syntax
                    highlighting
                  </li>
                  <li>
                    Fixed the append command's <var>scroll_to_end</var>{" "}
                    parameter sometimes not working
                  </li>
                  <li>
                    Fixed <i>Goto Symbol</i> sometimes being scrolled
                    incorrectly
                  </li>
                  <li>Fixed multi-select file limit applying to sidebar</li>
                  <li>Fixed auto-complete related hang in some large files</li>
                  <li>Linux: Fixed print sometimes not working</li>
                  <li>
                    Linux: Fixed wrong order of yes/no buttons in GTK dialogs
                  </li>
                  <li>Linux: Fixed letters sometimes being cut off</li>
                  <li>
                    Windows: Always make a new window when launching main
                    executable on Windows
                  </li>
                  <li>
                    Windows: Fixed window icon not scaling properly on Windows
                  </li>
                  <li>
                    Windows: Fixed globs not being expanded in some cases on
                    Windows
                  </li>
                  <li>
                    Mac: Fixed auto theme not changing with OS auto theme on
                    macOS
                  </li>
                </ul>
              </article>

              <article>
                <h3>4 (Build 4107)</h3>
                <div className="release-date">20 May 2021</div>
                <a href="">Show downloads</a>
                <div className="forum-link">
                  See also the{" "}
                  <a href="https://www.sublimetext.com/blog/articles/sublime-text-4">
                    Announcement Post
                  </a>
                </div>

                <h3>Release Highlights</h3>
                <ul className="topic">
                  <li>Multi-select tabs to view them side-by-side</li>
                  <li>
                    Context-aware auto complete by finding similar code
                    elsewhere in the current project
                  </li>
                  <li>
                    Symbols have kind information that is shown for completions
                    and navigation
                  </li>
                  <li>
                    Theme can follow system dark mode preference and title bars
                    can be themed on all platforms
                  </li>
                  <li>
                    Syntax highlighting now supports back-tracking and
                    inheritance
                  </li>
                  <li>
                    Many syntax highlighting improvements as well as builtin
                    TypeScript, JSX and TSX support
                  </li>
                  <li>
                    GPU rendering for improved performance. Enabled by default
                    on macOS
                  </li>
                  <li>ARM64 support for Linux and macOS (Apple Silicon)</li>
                  <li>
                    Many plugin API additions particularly to better support
                    plugins like LSP
                  </li>
                  <li>Python 3.8 support for plugins</li>
                </ul>

                <h3>GPU Rendering</h3>
                <ul className="topic">
                  <li>
                    New <var>hardware_acceleration</var> setting will composite
                    the UI on the GPU
                  </li>
                  <li>
                    By default, GPU rendering is enabled on Mac, and disabled on
                    Windows and Linux
                  </li>
                  <li>
                    Details about the active GPU will be displayed in the
                    Console
                  </li>
                </ul>

                <h3>Context-aware Auto Complete</h3>
                <ul className="topic">
                  <li>
                    The auto complete engine now suggests completions based on
                    patterns in existing code
                  </li>
                  <li>
                    Uses the entire project as a source, instead of just the
                    current view
                  </li>
                  <li>
                    Plugins may specify symbol kind info to be displayed in
                    suggestions list
                  </li>
                </ul>

                <h3>Tab Multi-Select</h3>
                <ul className="topic">
                  <li>
                    Multiple tabs can be selected using <var>ctrl/cmd</var>,
                    their contents will be shown side-by-side
                  </li>
                  <li>
                    Selecting multiple files from the sidebar will also preview
                    them simultaneously
                  </li>
                  <li>
                    Included themes have a tab connector joining the active
                    sheet and tab when using sheet multi-select
                  </li>
                  <li>
                    The sidebar can now select multiple files using{" "}
                    <var>alt</var>
                  </li>
                  <li>
                    Goto Anything allows opening tabs side-by-side using{" "}
                    <var>ctrl/cmd</var>
                  </li>
                  <li>
                    The Definition popup has a dedicated button for opening
                    files side-by-side
                  </li>
                  <li>
                    Multiple tabs can also be selected from the tab dropdown
                  </li>
                  <li>
                    The menu <i>Selection/Tab Selection</i> contains various
                    options for manipulating tab multi-select
                  </li>
                  <li>
                    <i>File/New View into File</i> has been replaced by{" "}
                    <i>File/Split View</i> using multi-select
                  </li>
                </ul>

                <h3>Python 3.8 API</h3>
                <ul className="topic">
                  <li>Added a Python 3.8 API environment for plugins</li>
                  <li>
                    Plugins can choose Python version via{" "}
                    <var>.python-version</var> file in plugin folder
                  </li>
                  <li>
                    Existing plugins are fully supported via legacy Python 3.3
                    API
                  </li>
                  <li>
                    Many API improvements and additions - see API section for
                    more details
                  </li>
                </ul>

                <h3>Goto Symbol</h3>
                <ul className="topic">
                  <li>
                    Goto Symbol in Project is now significantly faster on huge
                    projects
                  </li>
                  <li>
                    Icons are now shown next to symbols, indicating the symbol
                    kind
                  </li>
                  <li>Symbols with 3 characters or less are now indexed</li>
                </ul>

                <h3>Syntax Definitions</h3>
                <ul className="topic">
                  <li>
                    Added out of the box support for TypeScript, JSX and TSX -
                    thanks to{" "}
                    <a href="https://github.com/Thom1729">Thomas Smith</a>
                  </li>
                  <li>
                    Added ability to "branch" within syntax definitions, for
                    non-deterministic or multi-line constructs
                  </li>
                  <li>
                    Many syntax highlighting improvements, including significant
                    improvements to:
                    <ul>
                      <li>
                        <i>Erlang</i>, with thanks to{" "}
                        <a href="https://github.com/deathaxe">deathaxe</a>
                      </li>
                    </ul>
                  </li>
                  <li>
                    Significantly improved load times, match times and reduced
                    cache size on disk
                  </li>
                  <li>
                    <var>embed</var> is now lazy loaded, resulting in much
                    higher performance for syntaxes like markdown
                  </li>
                  <li>
                    Added <var>branch</var> and <var>fail</var> for
                    non-deterministic parsing
                  </li>
                  <li>
                    Added <var>version: 2</var> to fix edge cases while
                    retaining backwards compatibility
                  </li>
                  <li>
                    Added <var>extends</var> to inherit from another syntax
                    definition. Multiple inheritance is supported, provided all
                    parents have the same base syntax
                  </li>
                  <li>
                    Added <var>hidden_extensions</var>
                  </li>
                  <li>
                    Allow using <var>pop</var> alongside <var>push</var>/
                    <var>set</var>/<var>embed</var>/<var>branch</var>
                  </li>
                  <li>
                    Fixed a performance issue with bounded repeats in regular
                    expressions
                  </li>
                  <li>
                    Syntax tests can now assert that reindent is working as
                    expected
                  </li>
                  <li>Syntax tests can now assert that symbols are indexed</li>
                  <li>Prevent infinite include loops via with_prototype</li>
                  <li>Fixed a number of scope related bugs</li>
                  <li>Fixed some regex capture related bugs</li>
                  <li>
                    Added more information to the <i>Show Scope Name</i> popup
                  </li>
                </ul>

                <h3>OS Compatibility</h3>
                <ul className="topic">
                  <li>
                    The following operating systems are no longer supported as a
                    result of adding Python 3.8:
                    <ul style={{ listStyleType: "circle" }}>
                      <li>OS X 10.7</li>
                      <li>OS X 10.8</li>
                      <li>Windows XP</li>
                      <li>Windows Vista</li>
                    </ul>
                  </li>
                </ul>

                <h3>Platform Integration</h3>
                <ul className="topic">
                  <li>
                    Added automatic dark/light theme and color scheme switching,
                    based on OS theme changes
                  </li>
                  <li>
                    <var>subl -</var> can now be used to read from stdin on all
                    platforms
                  </li>
                  <li>
                    Windows will remember their Virtual Desktop/Space/Workspace,
                    controlled by the <var>remember_workspace</var> setting
                  </li>
                  <li>
                    Scroll bars now follow platform conventions when clicking on
                    them. Configurable using{" "}
                    <var>Scroll Bar.sublime-mousemap</var>
                  </li>
                  <li>
                    Mac: Releases use universal binaries with Apple Silicon
                    support
                  </li>
                  <li>Mac: Updated icon to follow macOS 11 style</li>
                  <li>
                    Mac: Windows will now stay maximized when using Mac window
                    tabs
                  </li>
                  <li>
                    Mac: Fix various issues with the wrong cursor being used
                  </li>
                  <li>Linux: ARM64 builds are now available</li>
                  <li>Linux: Text drag and drop is now supported</li>
                  <li>Linux: Added proper support for Wayland</li>
                  <li>Linux: Touch screen events are now handled</li>
                  <li>
                    Linux: Better support for copy+paste with other applications
                    that don't support utf8 text
                  </li>
                  <li>
                    Linux: Native file dialogs like those for KDE will be used
                    when configured
                  </li>
                  <li>Windows: IME preview and multi-select support</li>
                  <li>Windows, Linux: Added support for custom title bars</li>
                  <li>
                    Windows, Linux: Use vsync for animations instead of a fixed
                    60hz
                  </li>
                  <li>
                    Mac, Linux: Improved compatibility with some keyboard
                    layouts
                  </li>
                </ul>

                <h3>Application Behavior</h3>
                <ul className="topic">
                  <li>
                    Added <i>Safe Mode</i>, to simulate a clean install. Enabled
                    by passing <var>--safe-mode</var> on the command line or
                    holding <var>shift+alt</var>/<var>option</var> at startup on
                    Windows/macOS respectively
                  </li>
                  <li>
                    Added <i>Help/Report a Bug</i> to link to our public issue
                    tracker
                  </li>
                  <li>
                    Added options to <var>hot_exit</var> setting to control
                    behavior when the last window is closed
                  </li>
                  <li>
                    Fixed a possible case where an update loses the current
                    session
                  </li>
                  <li>
                    Settings containing a UTF-8 BOM will no longer fail to load
                  </li>
                  <li>Added support for previewing TGA and PSD images</li>
                  <li>
                    Added <var>close_deleted_files</var> setting to control
                    behavior of session restoration when files have been deleted
                    on disk
                  </li>
                  <li>
                    Popup windows now use virtual windows for improved
                    performance
                  </li>
                  <li>
                    Improved performance when loading files with very long lines
                  </li>
                  <li>Improved rendering performance on very long lines</li>
                  <li>Improved performance with large session files</li>
                  <li>
                    Data directories have dropped the "3", though if a "3"
                    directory still exists it will be used
                  </li>
                  <li>
                    Mac: <i>Quick Switch Project</i> now works without any
                    windows open
                  </li>
                  <li>
                    Mac, Linux: The cache and index are now located in the
                    proper location (<var>~/.cache</var> and{" "}
                    <var>~/Library/Caches</var> respectively)
                  </li>
                </ul>

                <h3>Auto Complete</h3>
                <ul className="topic">
                  <li>
                    Typing the full tab trigger of a snippet will move it to the
                    top of the results
                  </li>
                  <li>
                    Manually typing in the only available completion will hide
                    the auto complete popup
                  </li>
                  <li>
                    <var>.sublime-completion</var> files can now specify{" "}
                    <var>annotation</var>, <var>kind</var> and{" "}
                    <var>details</var>
                  </li>
                  <li>Ranking quality improvements</li>
                  <li>
                    Improved behavior of completions starting with non-word
                    characters
                  </li>
                  <li>
                    <var>auto_complete_trailing_symbols</var> is now disabled by
                    default
                  </li>
                  <li>
                    <var>cancelCompletion</var> will no longer prevent manual
                    invocation
                  </li>
                  <li>
                    Added the <var>auto_complete_when_likely</var> setting
                  </li>
                  <li>
                    Added <var>auto_complete_preserve_order</var> setting
                  </li>
                  <li>
                    Added <var>auto_complete_include_snippets_when_typing</var>{" "}
                    setting
                  </li>
                  <li>
                    Added <var>auto_complete_use_index</var> setting
                  </li>
                  <li>
                    Added <var>auto_complete_use_history</var> setting to
                    control if previous choices are automatically selected
                  </li>
                  <li>
                    Running the <var>auto_complete</var> command when auto
                    complete is already showing will re-query plugins for
                    results
                  </li>
                  <li>
                    <var>auto_complete_selector</var> now applies to the
                    position before the just-typed in character, matching{" "}
                    <var>auto_complete_triggers</var>
                  </li>
                </ul>

                <h3>Input Handling</h3>
                <ul className="topic">
                  <li>
                    Modifier key taps can now be used as part of a key binding.
                    For example, <var>["ctrl", "ctrl"]</var> will trigger when{" "}
                    <var>Ctrl</var> is pressed twice without pressing any other
                    keys in between
                  </li>
                  <li>
                    Linux: <var>AltGr</var> can now be used in key bindings via{" "}
                    <var>altgr</var>
                  </li>
                  <li>
                    Linux: Added a workaround for a touchscreen driver bug,
                    which would cause right click and mouse scrolling to stop
                    working
                  </li>
                  <li>
                    Linux: When the menu is hidden, pressing alt will show it
                  </li>
                  <li>Mac: Fix Pinyin input</li>
                  <li>Mac: Keypad keys can now be bound to as expected</li>
                  <li>Mac: Added key bindings for macOS application tabs</li>
                  <li>
                    Windows, Linux: Hide mouse cursor when typing. Controlled
                    via <var>hide_pointer_while_typing</var> setting
                  </li>
                  <li>
                    Windows, Linux: Fixed being unable to bind{" "}
                    <var>Ctrl+Break</var>
                  </li>
                </ul>

                <h3>Editor Control</h3>
                <ul className="topic">
                  <li>
                    Added <i>File/Print</i>, which prints via a browser
                  </li>
                  <li>
                    Added <i>Edit/Copy as HTML</i>
                  </li>
                  <li>
                    Build systems now use new annotations functionality instead
                    of phantoms, reducing re-flow
                  </li>
                  <li>Undo history is preserved in the session</li>
                  <li>
                    Comments and layout are preserved when programmatically
                    editing preferences
                  </li>
                  <li>
                    Caret blinking is disabled by default. Set{" "}
                    <var>caret_style</var> setting to <var>smooth</var> for
                    previous behavior
                  </li>
                  <li>Improved automatic indentation detection</li>
                  <li>
                    Added relative line numbers, controlled by the{" "}
                    <var>relative_line_numbers</var> setting
                  </li>
                  <li>
                    Added setting <var>scroll_context_lines</var>
                  </li>
                  <li>
                    Added setting <var>hide_pointer_while_typing</var>
                  </li>
                  <li>
                    Added setting <var>control_character_style</var>
                  </li>
                  <li>
                    Added <i>Project/Recent/Remove Deleted</i>
                  </li>
                  <li>
                    Added <var>chain</var> command to run multiple commands in
                    series
                  </li>
                  <li>
                    <var>switch_file</var> command now handles filenames with
                    compound extensions
                  </li>
                  <li>
                    The <var>scroll_past_end</var> setting now supports
                    customizing the scroll distance using numbers from{" "}
                    <var>0.0</var> to <var>1.0</var>
                  </li>
                  <li>
                    Double-clicking a semi-transient sheet's tab will now fully
                    open the sheet
                  </li>
                  <li>
                    <var>trim_trailing_white_space_on_save</var> can now be set
                    to <var>"not_on_caret"</var>
                  </li>
                  <li>
                    <var>trim_trailing_white_space_on_save</var> now trims only
                    newly inserted trailing whitespace by default. Controlled
                    via <var>trim_only_modified_white_space</var> setting
                  </li>
                  <li>
                    Expanded <var>draw_white_space</var> setting, supporting
                    leading and trailing white space
                  </li>
                  <li>
                    Unicode white space characters, such as the zero width
                    no-break space, are now drawn as hex values. Controlled via{" "}
                    <var>draw_unicode_white_space</var> setting
                  </li>
                  <li>
                    Fixed spelling correction to support languages with upper
                    case characters after start of word
                  </li>
                  <li>
                    Added commands to simplify customizing the active theme or
                    color scheme
                  </li>
                  <li>
                    <i>Quick Switch Project</i> will open the selected project
                    in a new window if <var>Ctrl</var> (<var>Cmd</var> on Mac)
                    is held down
                  </li>
                  <li>
                    Added <var>wrap_width_style</var> preference
                  </li>
                  <li>
                    Added <var>console_max_history_lines</var>
                  </li>
                  <li>
                    Added additional settings to control the status bar:{" "}
                    <var>show_sidebar_button</var>, <var>show_indentation</var>{" "}
                    and <var>show_syntax</var>
                  </li>
                  <li>
                    Console now uses Python syntax highlighting by default
                  </li>
                  <li>
                    Added <i>Central European (Mac)</i> encoding support
                  </li>
                  <li>
                    Key Bindings: <i>Join Lines</i> is now on{" "}
                    <var>Ctrl+Shift+J</var> / <var>Cmd+Shift+J</var>
                  </li>
                  <li>
                    Key Bindings: <i>Expand Selection to Indentation</i> is no
                    longer bound by default
                  </li>
                  <li>
                    Key Bindings: <var>Ctrl+J</var> / <var>Cmd+J</var> is now
                    used as a prefix for sequential key bindings, similar to{" "}
                    <var>Ctrl+K</var> / <var>Cmd+K</var>
                  </li>
                  <li>
                    Code Folding: fixed some edge-case incorrect behaviors
                  </li>
                  <li>Linux: Added support for alternate font weight names</li>
                  <li>
                    Linux: Selection is no longer cleared when another
                    application makes a selection
                  </li>
                  <li>
                    Linux: Added <var>Ctrl+Space</var> to trigger Auto Complete
                  </li>
                  <li>
                    Linux, Windows: Added <var>Alt+Shift+Left Mouse Button</var>{" "}
                    as an alternative column selection binding
                  </li>
                  <li>
                    Linux, Windows: Added Shift+F10 key binding to open the
                    context menu
                  </li>
                </ul>

                <h3>Text Commands</h3>
                <ul className="topic">
                  <li>Macros now record Find commands</li>
                  <li>
                    Reworked <i>Jump Back</i> and <i>Jump Forward</i> commands
                  </li>
                  <li>
                    Improved behavior of <i>Wrap Paragraph</i>
                  </li>
                  <li>
                    Improved behavior of <i>Swap Lines</i>
                  </li>
                  <li>
                    Added <i>Revert Diff Hunk</i>
                  </li>
                  <li>
                    Added <i>Selection/Expand Selection</i> as a general
                    mechanism to expand the selection
                  </li>
                  <li>
                    <i>Selection/Split into Lines</i> will now split a selection
                    into words if the selection doesn't contain any newlines
                  </li>
                  <li>
                    Show a sum in the status bar when there are multiple
                    selections and all of them are numbers
                  </li>
                  <li>
                    <var>set_file_type</var> command now accepts "scope:"
                    prefixed syntax names
                  </li>
                  <li>
                    Fixed <var>sort_lines</var> replacing unicode newlines with
                    regular ones
                  </li>
                </ul>

                <h3>Snippets</h3>
                <ul className="topic">
                  <li>
                    Added <var>auto_complete_include_snippets</var> setting, for
                    disabling auto complete integration
                  </li>
                  <li>
                    Added <var>ignored_snippets</var> setting, for disabling
                    default snippets
                  </li>
                </ul>

                <h3>Indexing (Goto Definition)</h3>
                <ul className="topic">
                  <li>
                    Files ignored by <var>.gitignore</var> are not indexed by
                    default. Controlled via <var>index_exclude_gitignore</var>{" "}
                    setting
                  </li>
                  <li>
                    Files without known extensions are no longer indexed by
                    default. Controlled via{" "}
                    <var>index_skip_unknown_extensions</var> setting
                  </li>
                  <li>Improved behavior with constantly changing files</li>
                  <li>Significantly improved load times</li>
                </ul>

                <h3>Files and Folders</h3>
                <ul className="topic">
                  <li>Saving files is now asynchronous</li>
                  <li>
                    Improved performance when adding directories with extreme
                    amounts of files
                  </li>
                  <li>
                    When <var>save_on_focus_lost</var> is enabled, closing an
                    unsaved file will save and close it, instead of prompting to
                    save
                  </li>
                  <li>
                    Improved behavior of <var>save_on_focus_lost</var> in
                    conjunction with administrator owned files
                  </li>
                  <li>
                    Added <var>reload_file_on_change</var> setting to control if
                    files are automatically reloaded or not
                  </li>
                  <li>
                    <var>folder_exclude_patterns</var> and{" "}
                    <var>folder_include_patterns</var> now support
                    project-relative paths, by starting the path with{" "}
                    <var>//</var>
                  </li>
                  <li>
                    Folders in the sidebar can be recursively expanded via
                    alt+arrow key
                  </li>
                  <li>
                    Added <var>preview_on_click</var> setting to support only
                    previewing files on left click
                  </li>
                  <li>
                    Windows: Fixed <i>Open Containing Folder</i> for UNC paths
                  </li>
                  <li>
                    Windows: Fixed Save dialog not showing for new files with
                    control characters on the first line
                  </li>
                  <li>
                    Windows: Fixed <var>Ctrl+Backspace</var> inserting a{" "}
                    <var>DEL</var> character when a dialog is open in the
                    background
                  </li>
                  <li>
                    Linux: Fix recreated directories not working correctly with
                    file change monitoring
                  </li>
                  <li>
                    Linux: Recursively expanding and collapsing sidebar folders
                    now works with <var>alt</var> or <var>super</var>
                  </li>
                </ul>

                <h3>Find</h3>
                <ul className="topic">
                  <li>
                    Find results are highlighted on the scroll bar, controlled
                    by <var>highlight_find_results_in_scrollbar</var>
                  </li>
                  <li>
                    Find in Selection now highlights the area that will be
                    searched
                  </li>
                  <li>
                    Commands can now be run without the find panel having input
                    focus
                  </li>
                  <li>Fix keypad enter not working in find panel</li>
                  <li>Improved find history behavior</li>
                  <li>
                    Find: Various performance improvements with large files
                    using graceful degradation
                  </li>
                  <li>
                    Find: Fixed adjacent matches being skipped when find in
                    selection is in use
                  </li>
                  <li>
                    Find: Fixed find in selection option not being cleared when
                    changing tabs
                  </li>
                  <li>
                    Find: Selection will no longer be reset after{" "}
                    <i>Find All</i> is used when finding in selection
                  </li>
                  <li>
                    Find: Results are now properly highlighted on{" "}
                    <i>Find All</i> when <var>close_find_after_find_all</var> is
                    turned off
                  </li>
                  <li>
                    Find in Files: Improved performance with large numbers of
                    matches
                  </li>
                  <li>
                    Find in Files: Can now filter by <var>.gitignore</var>
                  </li>
                  <li>
                    Find in Files: Added Preserve Case option for replacements
                  </li>
                  <li>
                    Find in Files: Fix not recursing into directories on
                    networked file systems
                  </li>
                  <li>Find in Files: Hide rulers by default in find results</li>
                  <li>
                    Find in Files: Added <i>Find/Cancel Find in Files</i> menu
                    item
                  </li>
                  <li>
                    Find in Files: Binary file patterns are applied when an
                    explicit folder is given
                  </li>
                  <li>
                    Find in Files: Using "Find in Folder…" from the sidebar
                    context menu will apply project file filters
                  </li>
                  <li>
                    Find in Files: Added <var>close_find_after_find_all</var>{" "}
                    preference
                  </li>
                  <li>
                    Find in Files: Added <var>close_find_after_replace_all</var>{" "}
                    preference
                  </li>
                </ul>

                <h3>UI</h3>
                <ul className="topic">
                  <li>
                    Changed default color scheme to <i>Mariana</i>
                  </li>
                  <li>Added Default Dark theme</li>
                  <li>
                    Added <var>themed_title_bar</var> setting
                  </li>
                  <li>
                    Changed file tab style, adding <var>file_tab_style</var>{" "}
                    setting
                  </li>
                  <li>Goto Symbol shows more information about symbols</li>
                  <li>
                    Definitions hover popup shows more information about symbols
                  </li>
                  <li>
                    Sheets without input focus are now dimmed when using
                    included themes
                  </li>
                  <li>
                    Added a New Tab button in the tab bar, and{" "}
                    <var>hide_new_tab_button</var> setting
                  </li>
                  <li>
                    Added <var>show_tab_close_buttons_on_left</var> setting
                  </li>
                  <li>
                    Added <var>highlight_gutter</var> and{" "}
                    <var>highlight_line_number</var> settings
                  </li>
                  <li>
                    Added the ability to auto hide the menu, tabs, and status
                    bar when typing. See <var>auto_hide_menu</var> and related
                    settings
                  </li>
                  <li>
                    Window title bar can be controlled by{" "}
                    <var>show_rel_path</var> and <var>show_project_first</var>{" "}
                    settings
                  </li>
                  <li>
                    Tab context menu now includes <i>Close Unmodified Files</i>{" "}
                    and <i>Close Deleted Files</i> entries
                  </li>
                  <li>
                    Side bar row highlights now properly reflect the selected
                    tabs
                  </li>
                  <li>
                    Color Schemes: Added <var>glow</var> font option
                  </li>
                  <li>
                    Color Schemes: Added support for the <var>underline</var>{" "}
                    font style
                  </li>
                  <li>
                    Color Schemes: Added new property,{" "}
                    <var>inactive_selection_border</var>
                  </li>
                  <li>
                    Color Schemes: Slightly darkened the background of Mariana
                  </li>
                  <li>
                    Color Schemes: Added support for stippled_underline and
                    squiggly_underline
                  </li>
                  <li>
                    Color Schemes: <var>.hidden-tmTheme</var> files are now
                    supported by the <var>.sublime-color-scheme</var> convertor
                  </li>
                  <li>
                    Themes: Included themes use variables extensively, making
                    customization simpler
                  </li>
                  <li>
                    Themes: Added <var>style</var> property for{" "}
                    <var>title_bar</var> element, for better integration with OS
                    "dark modes"
                  </li>
                  <li>
                    Themes: The <var>tree_row</var> for the file with input
                    focus now gets the attribute <var>highlighted</var>
                  </li>
                  <li>
                    Themes: The <var>settings</var> key now supports objects,
                    with keys being settings and values being a boolean, string
                    or array of strings
                  </li>
                  <li>
                    Themes: Added <var>sheet_contents</var> class to text, image
                    and HTML sheets
                  </li>
                  <li>
                    Themes: Added the <var>background_modifier</var> property
                    for <var>sheet_contents</var>
                  </li>
                  <li>
                    Themes: Added a number of new attributes to{" "}
                    <var>tab_control</var> for richer tab theming
                  </li>
                  <li>
                    Themes: The <var>highlighted</var> attribute is only applied
                    to the most recently active sheet, rather that the most
                    recently active sheet in each group
                  </li>
                  <li>
                    Themes: <var>tab_control</var> and <var>sheet_contents</var>{" "}
                    classes now synchronize their <var>highlighted</var> and{" "}
                    <var>hover</var> attributes
                  </li>
                  <li>
                    Themes: <var>tooltip</var> controls now support animations
                    to their opacity
                  </li>
                  <li>Linux: Show sequential key bindings in the menu</li>
                  <li>
                    Linux: Fixed context menu position being slightly offset
                  </li>
                </ul>

                <h3>Spell Checking</h3>
                <ul className="topic">
                  <li>Updated dictionaries</li>
                  <li>Added support for non-utf8 dictionaries</li>
                  <li>Updated Hunspell for improved suggestions</li>
                  <li>System dictionaries are now available on Linux</li>
                  <li>
                    Dictionaries in <var>~/Library/Spelling</var> are now
                    available on Mac
                  </li>
                </ul>

                <h3>Rendering</h3>
                <ul className="topic">
                  <li>
                    Properly query glyph extents in order to avoid cutting off
                    large glyphs
                  </li>
                  <li>
                    Windows, Linux: Added support for per-display subpixel
                    ordering
                  </li>
                  <li>Mac: Improved window resize performance</li>
                  <li>
                    Windows: Fixed rendering bug where other applications could
                    cause persistent artifacts via window animations
                  </li>
                  <li>Windows: Add support for color emoji</li>
                </ul>

                <h3>API</h3>
                <ul className="topic">
                  <li>Improved coverage of plugin profiling</li>
                  <li>The cProfile module is now included on Linux</li>
                  <li>
                    Added HTML sheets, which can be created via{" "}
                    <var>window.new_html_sheet()</var>
                  </li>
                  <li>
                    <var>repr</var> now works as expected
                  </li>
                  <li>Updated OpenSSL to 1.1.1j</li>
                  <li>
                    <var>.sublime-commands</var> files now support filtering
                    commands via the "platform" key
                  </li>
                  <li>
                    Minihtml now handles <var>list-style-type</var> CSS property
                    - <var>circle</var>, <var>square</var> and <var>disc</var>
                  </li>
                  <li>
                    Minihtml now processes <var>subl:</var> links, running them
                    as commands
                  </li>
                  <li>
                    Minihtml now supports <var>white-space: nowrap</var>
                  </li>
                  <li>Improved minihtml rendering performance</li>
                  <li>
                    <var>EventListener</var> has many new features and bug fixes
                    <ul style={{ listStyleType: "circle" }}>
                      <li>
                        <var>on_query_completions()</var> can now
                        <ul style={{ listStyleType: "square" }}>
                          <li>Return suggestions asynchronously</li>
                          <li>Return command completions</li>
                          <li>Include symbol kind information</li>
                        </ul>
                      </li>
                      <li>
                        New: <var>on_init()</var> is called once with list of
                        views opened before plugin was loaded
                      </li>
                      <li>
                        New: <var>on_exit()</var> is called immediately before{" "}
                        <var>plugin_host</var> exits, after API is shut down
                      </li>
                      <li>
                        New: <var>on_text_changed()</var> and{" "}
                        <var>on_text_changed_async()</var> provide detailed
                        modification info
                      </li>
                      <li>
                        New methods:
                        <ul style={{ listStyleType: "square" }}>
                          <li>
                            <var>on_reload()</var>
                          </li>
                          <li>
                            <var>on_revert()</var>
                          </li>
                          <li>
                            <var>on_new_window()</var>
                          </li>
                          <li>
                            <var>on_new_window_async()</var>
                          </li>
                          <li>
                            <var>on_pre_close_window()</var>
                          </li>
                          <li>
                            <var>on_pre_move()</var>
                          </li>
                          <li>
                            <var>on_post_move()</var>
                          </li>
                          <li>
                            <var>on_post_move_async()</var>
                          </li>
                          <li>
                            <var>on_new_project()</var>
                          </li>
                          <li>
                            <var>on_new_project_async()</var>
                          </li>
                          <li>
                            <var>on_load_project()</var>
                          </li>
                          <li>
                            <var>on_load_project_async()</var>
                          </li>
                          <li>
                            <var>on_pre_save_project()</var>
                          </li>
                          <li>
                            <var>on_post_save_project()</var>
                          </li>
                          <li>
                            <var>on_post_save_project_async()</var>
                          </li>
                          <li>
                            <var>on_pre_close_project()</var>
                          </li>
                          <li>
                            <var>on_associate_buffer()</var>
                          </li>
                        </ul>
                      </li>
                      <li>
                        Fixed <var>on_selection_modified()</var> to not be
                        called twice when left clicking
                      </li>
                      <li>
                        Fixed <var>on_selection_modified()</var> begin called on
                        non-selection state changes
                      </li>
                    </ul>
                  </li>
                  <li>
                    New <var>TextChangeListener</var> for getting callbacks when
                    a text buffer is changed
                    <ul style={{ listStyleType: "circle" }}>
                      <li>
                        Can be dynamically bound to a <var>Buffer</var> using{" "}
                        <var>TextChangeListener.attach</var> and{" "}
                        <var>TextChangeListener.detach</var>
                      </li>
                      <li>
                        Methods:
                        <ul style={{ listStyleType: "square" }}>
                          <li>
                            <var>on_text_changed()</var>
                          </li>
                          <li>
                            <var>on_text_changed_async()</var>
                          </li>
                          <li>
                            <var>on_revert()</var>
                          </li>
                          <li>
                            <var>on_revert_async()</var>
                          </li>
                          <li>
                            <var>on_reload()</var>
                          </li>
                          <li>
                            <var>on_reload_async()</var>
                          </li>
                        </ul>
                      </li>
                    </ul>
                  </li>
                  <li>
                    <var>Sheet</var> has some new methods:
                    <ul style={{ listStyleType: "circle" }}>
                      <li>
                        <var>file_name()</var>
                      </li>
                      <li>
                        <var>group()</var>
                      </li>
                      <li>
                        <var>close()</var>
                      </li>
                      <li>
                        <var>is_semi_transient()</var>
                      </li>
                      <li>
                        <var>Sheet.is_transient</var>
                      </li>
                    </ul>
                  </li>
                  <li>
                    <var>View</var> has a number of changes and improvements
                    <ul style={{ listStyleType: "circle" }}>
                      <li>
                        Fixed newly created views not having a valid viewport
                        before being returned to the API
                      </li>
                      <li>
                        <var>add_regions()</var> now has an{" "}
                        <var>annotations</var> parameter, to allow adding a
                        per-region annotation to the buffer. The <var>exec</var>{" "}
                        command uses this API for build errors
                      </li>
                      <li>
                        <var>add_regions()</var> calls that add an underline now
                        have that underline applied to whitespace characters
                      </li>
                      <li>
                        <var>show()</var> now takes <var>keep_to_left</var> and{" "}
                        <var>animate</var> parameters
                      </li>
                      <li>
                        <var>text_point()</var> and related functions now accept
                        a <var>clamp_column</var> parameter
                      </li>
                      <li>
                        Added <var>"glow"</var> to <var>style_for_scope()</var>
                      </li>
                      <li>
                        Added <var>View.rowcol_utf8()</var>,{" "}
                        <var>View.rowcol_utf16()</var>,{" "}
                        <var>View.text_point_utf8()</var> and{" "}
                        <var>View.text_point_utf16()</var>
                      </li>
                      <li>
                        Added <var>sublime.KEEP_ON_SELECTION_MODIFIED</var>,
                        which can be passed to <var>show_popup()</var>
                      </li>
                      <li>
                        New: <var>element()</var> returns a string describing
                        widget views (find input, quick panel input, etc)
                      </li>
                      <li>
                        New: <var>assign_syntax()</var> sets the syntax used in
                        a view, supports <var>Syntax</var> objects, paths and
                        scope selectors
                      </li>
                      <li>
                        New: <var>syntax()</var> returns the currently set{" "}
                        <var>Syntax</var>
                      </li>
                      <li>
                        New: <var>clones()</var>
                      </li>
                      <li>
                        New: <var>sheet()</var> and <var>sheet_id()</var>
                      </li>
                      <li>
                        New: <var>export_to_html()</var>
                      </li>
                    </ul>
                  </li>
                  <li>
                    <var>Window</var> has some changes and improvements:
                    <ul style={{ listStyleType: "circle" }}>
                      <li>
                        <var>show_quick_panel</var> now accepts placeholder
                        text, via the <var>placeholder</var> argument
                      </li>
                      <li>
                        Added <var>sublime.CLEAR_TO_RIGHT</var> and{" "}
                        <var>sublime.SEMI_TRANSIENT</var> for use with{" "}
                        <var>open_file()</var>
                      </li>
                      <li>
                        Added the flag <var>sublime.REPLACE_MRU</var> for use
                        with <var>open_file()</var>. When multiple sheets are
                        selected, this flag will cause the opened file to
                        replace the most recently used sheet with the file being
                        opened
                      </li>
                      <li>
                        Added the flag <var>sublime.WANT_EVENT</var> for use
                        with <var>show_quick_panel()</var>. This will pass an
                        event dict to the on_select callback. The dict will
                        contain the key modifier_keys, which will be a dict that
                        may contain zero or more of the keys: primary, ctrl,
                        super, alt, altgr
                      </li>
                      <li>
                        <var>open_file</var> now accepts{" "}
                        <var>sublime.ADD_TO_SELECTION</var> as a flag
                      </li>
                      <li>
                        New: <var>selected_sheets()</var>,{" "}
                        <var>selected_sheets_in_group()</var> and{" "}
                        <var>select_sheets()</var>
                      </li>
                      <li>
                        New: <var>workspace_file_name()</var>
                      </li>
                      <li>
                        New: <var>bring_to_front()</var>
                      </li>
                    </ul>
                  </li>
                  <li>
                    <var>sublime.ok_cancel_dialog()</var> and{" "}
                    <var>sublime.yes_no_cancel_dialog()</var> now accept an
                    optional <var>title</var> parameter
                  </li>
                  <li>
                    Added <var>sublime.open_dialog</var>,{" "}
                    <var>sublime.save_dialog</var> and{" "}
                    <var>sublime.select_folder_dialog</var>
                  </li>
                  <li>
                    Syntax definitions can be queried via{" "}
                    <var>sublime.list_syntaxes()</var>,{" "}
                    <var>sublime.find_syntax()</var>,{" "}
                    <var>sublime.syntax_from_path()</var>,{" "}
                    <var>sublime.find_syntax_by_name()</var>,{" "}
                    <var>sublime.find_syntax_by_scope()</var>, and{" "}
                    <var>sublime.find_syntax_for_file()</var>. They return{" "}
                    <var>Syntax</var> objects
                  </li>
                  <li>
                    Improved <var>sys.stdout</var> to extend{" "}
                    <var>io.TextIOBase</var>
                  </li>
                  <li>
                    <var>sublime.executable_path()</var>,{" "}
                    <var>sublime.packages_path()</var>,{" "}
                    <var>sublime.installed_packages_path()</var> and{" "}
                    <var>sublime.cache_path()</var> may now be called at import
                    time
                  </li>
                  <li>
                    Added <var>sublime.SymbolRegion</var> and{" "}
                    <var>sublime.SymbolLocation</var> with corresponding methods
                    on <var>View</var> and <var>Window</var>
                  </li>
                  <li>
                    Fix a bug with popup being stuck open when a popup is shown
                    in the hide event handler of another popup
                  </li>
                  <li>
                    Added <var>open_project_or_workspace</var> command
                  </li>
                  <li>
                    <var>append</var> command has new, optional{" "}
                    <var>disable_tab_translation</var> argument
                  </li>
                  <li>
                    Added <var>modifier_keys</var> to event dicts when commands
                    are invoked via a menu
                  </li>
                  <li>
                    Added <var>sublime.DYNAMIC_COMPLETIONS</var>.{" "}
                    <var>on_query_completions()</var> can return this flag to
                    indicate that completion results should be re-queried as the
                    user types
                  </li>
                  <li>
                    Added <var>sublime.INHIBIT_REORDER</var>. Returned by{" "}
                    <var>on_query_competions()</var>
                  </li>
                  <li>
                    <var>CompletionItem</var> now accepts a <var>details</var>{" "}
                    parameter, which can include basic HTML
                  </li>
                  <li>
                    <var>CommandInputHandler</var> now has an{" "}
                    <var>initial_selection()</var> method
                  </li>
                  <li>
                    Added <var>Region.to_tuple</var> and{" "}
                    <var>Phantom.to_tuple</var>
                  </li>
                  <li>
                    Fixed <var>ViewEventListener.on_load_async()</var> sometimes
                    not being called
                  </li>
                  <li>
                    Added <var>sublime.QuickPanelItem()</var> with support for
                    kind info, annotations and basic minihtml
                  </li>
                  <li>
                    Plugins may now add selections to the <i>Jump Back</i>{" "}
                    history list via the <var>add_jump_record</var> command
                  </li>
                  <li>
                    Plugins may suppress selections from the <i>Jump Back</i>{" "}
                    history list via the <var>jump_ignore_selection</var> region
                  </li>
                  <li>
                    Plugins may now disable the default HTML and CSS completions
                  </li>
                  <li>
                    Added <var>Buffer.id()</var> and{" "}
                    <var>Buffer.file_name()</var>
                  </li>
                  <li>
                    The <var>TextInputHandler</var> and{" "}
                    <var>ListInputHandler</var> classes may define a method{" "}
                    <var>want_event()</var> that returns <var>True</var> to
                    receive an extra parameter, an event dict, when the{" "}
                    <var>validate()</var> and <var>confirm()</var> methods are
                    called. The dict will contain the key modifier_keys, which
                    will be a dict that may contain zero or more of the keys:
                    primary, ctrl, super, alt, altgr
                  </li>
                  <li>
                    Add <var>sublime.ui_info()</var> for high-level information
                    about the UI
                  </li>
                  <li>
                    Popups will be properly positioned when displayed near the
                    right-hand side of the editor
                  </li>
                  <li>
                    Popups near the right-hand side of the editor with wrapped
                    lines will now be properly sized
                  </li>
                  <li>
                    Added <var>ListInputItem</var> so that{" "}
                    <var>ListInputHandler</var> objects can provide kind info,
                    annotations and details
                  </li>
                  <li>
                    Improvements to the API, applied to the new Python 3.8
                    environment only:
                    <ul style={{ listStyleType: "circle" }}>
                      <li>
                        <var>bool(sublime.Selection())</var> will return{" "}
                        <var>False</var> when <var>len() == 0</var>
                      </li>
                      <li>
                        <var>sublime.load_binary_resource()</var> now returns{" "}
                        <var>bytes</var> instead of <var>bytearray</var>
                      </li>
                      <li>
                        Added <var>Selection.__iter__()</var>
                      </li>
                      <li>
                        Added <var>Region.__iter__()</var>
                      </li>
                      <li>
                        Added <var>Region.__contains__()</var>
                      </li>
                      <li>
                        Added <var>Settings.to_dict()</var>
                      </li>
                      <li>
                        <var>Settings</var> can now be treated like a{" "}
                        <var>dict</var>
                      </li>
                      <li>
                        Plugins starting with <var>_</var> will be ignored,{" "}
                        <var>__all__</var> global will be respected
                      </li>
                      <li>
                        Events won't be reported until{" "}
                        <var>plugin_loaded()</var> has been called
                      </li>
                      <li>
                        <var>.pyc</var> files can now be imported when contained
                        within <var>.sublime-package</var> files, although they
                        will not be scanned for plugins
                      </li>
                    </ul>
                  </li>
                  <li>
                    The <var>certifi</var> Python package is preinstalled
                  </li>
                  <li>
                    Significant performance improvements when rapidly printing
                    to the Console
                  </li>
                  <li>
                    Added <var>sublime.log_control_tree()</var>. When enabled,
                    clicking with ctrl+alt will log the control tree under the
                    mouse to the console
                  </li>
                  <li>
                    Added <var>sublime.log_fps()</var>. When enabled, the render
                    times are tracked and logged
                  </li>
                  <li>
                    Added logging status functions:
                    <ul>
                      <li>
                        <var>sublime.get_log_commands()</var>
                      </li>
                      <li>
                        <var>sublime.get_log_input()</var>
                      </li>
                      <li>
                        <var>sublime.get_log_build_systems()</var>
                      </li>
                      <li>
                        <var>sublime.get_log_result_regex()</var>
                      </li>
                      <li>
                        <var>sublime.get_log_indexing()</var>
                      </li>
                      <li>
                        <var>sublime.get_log_fps()</var>
                      </li>
                      <li>
                        <var>sublime.get_log_control_tree()</var>
                      </li>
                    </ul>
                  </li>
                  <li>
                    Logging functions are now toggle when no argument is passed:
                    <ul>
                      <li>
                        <var>sublime.log_commands()</var>
                      </li>
                      <li>
                        <var>sublime.log_input()</var>
                      </li>
                      <li>
                        <var>sublime.log_build_systems()</var>
                      </li>
                      <li>
                        <var>sublime.log_result_regex()</var>
                      </li>
                      <li>
                        <var>sublime.log_indexing()</var>
                      </li>
                      <li>
                        <var>sublime.log_fps()</var>
                      </li>
                      <li>
                        <var>sublime.log_control_tree()</var>
                      </li>
                    </ul>
                  </li>
                  <li>
                    Backwards Compatibility Break: The event parameter passed to
                    commands when a minihtml link is clicked changed from a
                    two-element list to a dict with the keys x and y
                  </li>
                </ul>
              </article>
            </section>
          </div>

          <aside>
            <div id="callout">
              <a href="https://www.sublimemerge.com">
                <span className="subhead">Introducing our Git client</span>
                <br />
                <b>
                  <img
                    src="https://www.sublimetext.com/images/merge_icon.svg"
                    className="icon"
                  />{" "}
                  Sublime Merge
                </b>
                <img
                  src="https://www.sublimetext.com/images/merge_callout_osx@2x.png"
                  id="merge-screenshot"
                  style={{ backgroundColor: "transparent" }}
                />
              </a>
            </div>

            <p>
              For notification about new releases, follow{" "}
              <a href="https://twitter.com/sublimehq">@sublimehq</a> on twitter.
            </p>

            <h3>Other Downloads</h3>
            <ul>
              <li>
                <a href="/dev">Dev Builds</a>
              </li>
              <li>
                <a href="/3">Sublime Text 3</a>
              </li>
              <li>
                <a href="/2">Sublime Text 2</a>
              </li>
            </ul>
          </aside>
        </section>
      </main>
      <footer>
        <section>
          <div className="footer_start"></div>
        </section>
      </footer>
    </body>
  );
}

export default App;
