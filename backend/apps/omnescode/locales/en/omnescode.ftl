oc-pane-dashboard = Dashboard
oc-pane-config = Config
oc-pane-doctor = Doctor
oc-pane-code = Code
oc-pane-chat = Chat
oc-pane-logs = Logs
oc-pane-quickstart = Quickstart
oc-pane-sop = SOPs
oc-sop-empty-read-only = (no nodes)
oc-sop-runs-stale = SOP run status is stale. Retrying…
oc-chrome-summary-sessions = sessions
oc-chrome-summary-ram = ram
oc-chrome-summary-cpu = cpu
oc-chrome-summary-loading = loading

oc-app-help-cycle-mode = Cycle mode
oc-app-help-help = Help
oc-app-help-reload = Reload daemon
oc-app-help-quit = Quit
oc-app-keybindings-title = Keybindings
oc-app-help-filter-label = Filter
oc-app-help-filter-placeholder = type a key or action…
oc-app-help-no-matches = No matching keybindings
oc-app-help-controls = { $cancel } clear/close · { $up }/{ $down } scroll · Type to filter

oc-app-reload-line-1 = The daemon process stays running (same PID), but every
oc-app-reload-line-2 = subsystem tears down and re-initializes from the on-disk
oc-app-reload-line-3 = config:
oc-app-reload-bullet-gateway = { "  " }• Gateway listener stops and rebinds
oc-app-reload-bullet-channels = { "  " }• Channel listeners (Matrix, Slack, etc.) respawn
oc-app-reload-bullet-mcp = { "  " }• MCP servers, scheduler, heartbeat re-init
oc-app-reload-bullet-provider = { "  " }• Provider clients pick up new API keys / model defaults
oc-app-reload-socket-note = The RPC socket will briefly drop. The TUI will reconnect.
oc-app-quit-prompt = Quit omnescode?
oc-app-quit-explainer = The TUI closes. The daemon keeps running; reconnect anytime.
oc-app-reload-status-signalled = Daemon reload signalled — reconnecting…
oc-app-reload-confirm-row = { $confirm_chord } = reload   { $cancel_chord } = cancel
oc-error-daemon-version-mismatch = Version mismatch: omnescode is { $client_version } but the daemon is { $server_version }. Rebuild and restart the daemon from the same checkout as omnescode.
oc-error-daemon-initialize-timeout = omnescode connected to the daemon, but initialization did not finish within { $seconds }s. Restart the daemon and try again.
oc-error-spawned-daemon-startup = omnescode started an ephemeral daemon, but it failed to become ready: { $details }

oc-omnescode-tab-theme = Theme
oc-omnescode-tab-agent-theme = Agent Themes
oc-omnescode-tab-presets = Presets
oc-omnescode-tab-bindings = Keybindings
oc-omnescode-tab-locale = Locale
oc-omnescode-tab-connection = Connection
oc-omnescode-tab-todo-tracker = Todo tracker
oc-omnescode-conn-title = Connection ([wss] — Enter to edit)
oc-omnescode-conn-uri = WSS URI
oc-omnescode-conn-skip-verify = TLS skip verify
oc-omnescode-conn-skip-verify-routes = Skip-verify routes
oc-omnescode-conn-unset = unset
oc-omnescode-conn-no-routes = none
oc-omnescode-conn-saved = Saved
oc-omnescode-conn-edit-text = Enter to save, Esc to cancel.
oc-omnescode-conn-edit-bool = Enter toggles; this field saves on toggle.
oc-omnescode-conn-edit-routes = One route per line. Enter for a new line, Ctrl+S to save, Esc to cancel.
oc-omnescode-tracker-title = Todo tracker ([todotracker] — Enter to edit)
oc-omnescode-tracker-enabled = Enabled
oc-omnescode-tracker-enabled-at-start = Visible at start
oc-omnescode-tracker-location = Location
oc-omnescode-tracker-width = Side panel width
oc-omnescode-tracker-max-height = Bottom max height
oc-omnescode-tracker-saved = Saved. New Code sessions will use this.
oc-omnescode-tracker-saved-env-override = Saved to disk, but a OMNESCODE_todotracker__* environment override is active, so new sessions will use the override value until it is unset.
oc-omnescode-tracker-saved-resolve-error = Saved to disk, but resolving the effective settings failed (check OMNESCODE_todotracker__* overrides); new sessions may not use this value until it is fixed.
oc-omnescode-tracker-load-error = [todotracker] unreadable: { $error } — fix or delete the section in omnescode-config.toml by hand, then reopen this pane.
oc-omnescode-tracker-saved-still-invalid = Saved, but [todotracker] is still invalid — the other dimension must be greater than zero too.
oc-omnescode-tracker-edit-refused = Edit refused — repair [todotracker] first (see above).
oc-omnescode-tracker-edit-number = Enter to save, Esc to cancel.
oc-omnescode-tracker-edit-bool = Enter toggles; this field saves on toggle.
oc-omnescode-tracker-edit-location = Enter cycles bottom / left / right.
oc-omnescode-config-invalid-number = Enter a valid positive whole number.
oc-omnescode-config-positive-required = Numeric values must be greater than zero.
oc-omnescode-config-save-failed = Save failed: { $error }
oc-omnescode-config-save-mismatch = Saved values differ from what new sessions will use.
oc-omnescode-help-todo-tracker = edit todo tracker field
oc-omnescode-locale-loading = loading locales…
oc-omnescode-locale-download = ⬇ Download selected locale file
oc-omnescode-locale-set = Locale set to { $locale }. Download its locale file if needed, then restart to apply.
oc-omnescode-locale-fetching = Downloading locale files for { $locale }…
oc-omnescode-locale-downloaded = Downloaded { $written } for { $locale }. Skipped: { $skipped }
oc-omnescode-locale-fetch-failed = Locale download failed for { $locale }: { $err }
oc-omnescode-locale-list-failed = Failed to load locale list: { $err }
oc-omnescode-locale-pick-first = Select a locale row first, then download its locale file.
oc-omnescode-help-locale = select / download locale
oc-omnescode-help-conn = edit connection field

oc-omnescode-capture-prompt = Press a key combination…
oc-omnescode-capture-modal-title = Assign Key
oc-omnescode-hint-cancel = { $keys } to cancel

oc-omnescode-capture-assign = Assign as the new binding
oc-omnescode-capture-cancel = Cancel capture

oc-omnescode-help-switch-pane = Switch pane (Theme/Presets/Keybindings)
oc-omnescode-help-navigate = Navigate
oc-omnescode-help-apply-theme = Apply theme (live + saved)
oc-omnescode-help-assign-agent-theme = Assign theme to the selected agent
oc-omnescode-help-pick-agent = Choose theme for this agent
oc-omnescode-help-clear-agent-theme = Clear the agent's override
oc-omnescode-help-apply-preset = Apply preset (overwrites keybindings)
oc-omnescode-help-rebind = Rebind selected action
oc-omnescode-help-reset-default = Reset action to default
oc-omnescode-help-mouse-label = Mouse
oc-omnescode-help-mouse-desc = Click pane / row, scroll, click section tab
oc-omnescode-help-choose-section = Choose section
oc-omnescode-help-open-section = Open section
oc-omnescode-help-navigate-rows = Navigate rows
oc-omnescode-help-back-to-sections = Back to sections

oc-omnescode-agent-modal-title = Select Agent
oc-omnescode-agent-modal-pick = Pick agent, then choose a theme
oc-omnescode-agent-theme-loading = Loading agents…
oc-omnescode-agent-theme-no-agents = No agents configured
oc-omnescode-agent-theme-none = No override on this agent
oc-omnescode-agent-theme-set = Override set: { $agent } → { $theme }
oc-omnescode-agent-theme-cleared = Override cleared for { $agent }
oc-omnescode-agent-theme-hint = { $assign } assign theme   ·   { $clear } clear override

oc-input-no-pending-attachments = No pending attachments.
oc-input-no-clipboard-image = Clipboard is empty.
oc-input-placeholder-chat = Type to chat

oc-input-help-completions-navigate = Navigate completions
oc-input-help-completions-accept = Accept
oc-input-help-completions-dismiss = Dismiss


oc-input-attached = Attached: { $label }
oc-input-attach-error = Attach error: { $error }
oc-input-detached = Detached: { $name }
oc-input-invalid-index = Invalid index: { $index }
oc-input-pending-attachments-header = Pending attachments:
oc-input-attachment-manager-title = Pending attachments ({ $count })
oc-input-attachment-manager-hint = { $navigate } select · { $remove } remove · { $close } close · /detach N
oc-input-help-attachment-remove = Remove attachment
oc-input-help-attachment-detach = Remove by index
oc-input-clipboard-error = Clipboard error: { $error }
oc-input-clipboard-cleanup-error = Clipboard cleanup warning: { $count } temporary file(s) could not be removed and remain on disk.

oc-queue-empty = Nothing to send.
oc-cancel-timed-out = Cancel timed out; turn settled locally.
oc-turn-no-output = Turn completed with no output.
oc-queue-full = Queue is full ({ $cap } max). Wait for messages to send.
oc-queue-title = Queue ({ $count })
oc-queue-empty-list = No queued messages.
oc-queue-paused-ghost = Queue paused — press { $key } or send a message to resume
oc-queue-item-injected = (inject)
oc-queue-resumed = Queue resumed.
oc-queue-clear-empty = Queue is already empty.
oc-queue-cleared-all = Cleared { $count } queued message(s).
oc-queue-cleared-one = Cleared queued message { $index }.
oc-queue-clear-invalid = No queued message at index { $index } (queue has { $count }).
oc-queue-help-resume = Pause/resume queue
oc-queue-help-nav = Select queued
oc-queue-help-delete = Delete queued
oc-queue-help-clear = Clear queue ([N] = position)
oc-queue-help-edit = Edit queued
oc-queue-help-resize = Resize queue
oc-queue-help-enqueue = Queue message
oc-queue-help-inject = Send now (skip queue)
oc-queue-edit-busy = Finish or clear the current message before editing a queued one.
oc-queue-dispatch-failed = Could not send queued message: { $error }

oc-logs-label-timestamp = Timestamp
oc-logs-label-severity = Severity
oc-logs-label-category = Category
oc-logs-label-action = Action
oc-logs-label-outcome = Outcome
oc-logs-label-duration = Duration
oc-logs-section-message = Message
oc-logs-section-trace = Trace
oc-logs-section-attribution = Attribution
oc-logs-section-attributes = Attributes
oc-logs-preview-only = Full payload unavailable — showing preview fields only.
oc-logs-persisted-path = Persisted log: { $path }
oc-logs-no-event-selected = No event selected
oc-logs-loading = Loading…
oc-logs-search-action-apply = apply
oc-logs-search-action-cancel = cancel

oc-logs-help-mouse-label = Mouse
oc-logs-help-mouse-desc = Click rows, drag to select text, scroll wheel, double-click detail
oc-logs-copy = Copy
oc-logs-copied = [Copied]

oc-doctor-title = Doctor
oc-doctor-loading = Loading diagnostics...
oc-doctor-error = Doctor failed: { $error }
oc-doctor-error-unsupported-daemon = Doctor RPC is not available on this daemon yet. Restart the daemon with a build that includes doctor/run.
oc-doctor-no-results = No diagnostics yet
oc-doctor-summary = { $ok } ok  { $warnings } warnings  { $errors } errors
oc-doctor-filter-status = filter: { $filter }
oc-doctor-filter-all = All
oc-doctor-filter-problems = Problems
oc-doctor-filter-errors = Errors
oc-doctor-list-title = Diagnostics ({ $filter })
oc-doctor-detail-title = Detail
oc-doctor-no-selection = No diagnostic selected
oc-doctor-label-message = Message
oc-doctor-help-mouse = Mouse: click filter/select, scroll wheel
oc-doctor-error-daemon-timeout = The doctor check timed out. The daemon may be busy, unreachable, or processing a long-running request. Try again or check daemon connectivity.
oc-doctor-partial-banner = ⚠ Partial results — model probing timed out
oc-doctor-partial-hint = Some provider catalogs could not be reached. Results from config, workspace, and daemon checks are shown below. Press the refresh key to retry.

oc-dashboard-tab-overview = Overview
oc-dashboard-tab-sessions = Sessions
oc-dashboard-tab-agents = Agents
oc-dashboard-tab-memories = Memories
oc-dashboard-tab-health = Health
oc-dashboard-tab-cost = Cost
oc-dashboard-tab-cron = Cron

oc-dashboard-memory-not-configured = Memory is not configured yet. Use Quickstart or Config to add a memory backend, or ignore this tab until you need persistent memory.
oc-dashboard-search-action-apply = apply
oc-dashboard-search-action-cancel = cancel
oc-dashboard-search-prefix = search:
oc-dashboard-agent-rename-action-apply = rename
oc-dashboard-agent-rename-action-cancel = cancel
oc-dashboard-agent-rename-hint = rename
oc-dashboard-agent-rename-prefix = rename:
oc-dashboard-agent-rename-empty = Agent name cannot be empty
oc-dashboard-agent-rename-unchanged = Agent name unchanged
oc-dashboard-agent-rename-success = Renamed { $from } to { $to }
oc-dashboard-agent-rename-success-warnings = Renamed { $from } to { $to } with warnings: { $warnings }
oc-dashboard-agent-rename-failed = Rename failed: { $error }

oc-dashboard-label-connected = Connected
oc-dashboard-label-daemon = Daemon
oc-dashboard-label-socket = Socket
oc-dashboard-label-server = Server
oc-dashboard-label-protocol = RPC API
oc-dashboard-label-sessions = Sessions
oc-dashboard-label-config = Config
oc-dashboard-label-config-file = File
oc-dashboard-label-endpoint = Endpoint
oc-dashboard-label-workspace = Workspace
oc-dashboard-label-code-cwd = Code CWD
oc-dashboard-label-chat-cwd = Chat CWD
oc-dashboard-label-daemon-memory = Daemon Memory
oc-dashboard-label-daemon-cpu = Daemon CPU
oc-dashboard-label-insecure-tls = ⚠ unverified TLS — certificate not checked
oc-dashboard-label-uptime = Uptime
oc-dashboard-label-pid = PID
oc-dashboard-config-kind-default = default
oc-dashboard-config-kind-custom = custom
oc-dashboard-config-kind-temporary = temporary
oc-dashboard-daemon-local = local
oc-dashboard-daemon-remote = remote
oc-dashboard-cpu-with-cores = { $cpu } ({ $cores } cores)

oc-dashboard-no-tuis = No TUIs connected
oc-dashboard-no-session = No session selected
oc-dashboard-no-agent = No agent selected
oc-dashboard-no-entry = No entry selected
oc-dashboard-no-job = No job selected

oc-dashboard-detail-key = Key
oc-dashboard-detail-agent = Agent
oc-dashboard-detail-channel = Channel
oc-dashboard-detail-name = Name
oc-dashboard-detail-messages = Messages
oc-dashboard-detail-created = Created
oc-dashboard-detail-activity = Activity
oc-dashboard-detail-alias = Alias
oc-dashboard-detail-enabled = Enabled
oc-dashboard-detail-category = Category
oc-dashboard-detail-namespace = Namespace
oc-dashboard-detail-timestamp = Timestamp
oc-dashboard-detail-score = Score
oc-dashboard-detail-importance = Importance
oc-dashboard-detail-session = Session
oc-dashboard-detail-daily = Daily
oc-dashboard-detail-monthly = Monthly
oc-dashboard-detail-tokens = Tokens
oc-dashboard-detail-requests = Requests
oc-dashboard-detail-schedule = Schedule
oc-dashboard-detail-next-run = Next Run
oc-dashboard-detail-last-run = Last Run
oc-dashboard-detail-last-status = Last Status
oc-dashboard-detail-sessions = Sessions
oc-dashboard-detail-persisted-sessions = Persisted Sessions
oc-dashboard-cost-not-available = Cost tracking is not available. Configure a cost backend to see usage data.
oc-dashboard-cost-org-error = Organization billing snapshot is present but could not be read. Check the org_cost.json sync.

oc-dashboard-message-history = Message History ({ $count })
oc-dashboard-loading-messages = Loading messages…
oc-dashboard-loading = Loading…

oc-dashboard-section-channels = Channels
oc-dashboard-section-content = Content
oc-dashboard-section-process = Process
oc-dashboard-section-components = Components
oc-dashboard-section-details = Details
oc-dashboard-section-summary = Summary
oc-dashboard-section-by-period = By Period (your account)
oc-dashboard-section-org = Organization (billed)
oc-dashboard-period-today = Today
oc-dashboard-period-month = Month
oc-dashboard-period-quarter-prefix = Q
oc-dashboard-period-ytd = YTD
oc-dashboard-col-period = period
oc-dashboard-col-cost = cost($)
oc-dashboard-col-paid-tok = paid tok
oc-dashboard-col-free-tok = free tok
oc-dashboard-col-reqs = reqs
oc-dashboard-org-tok = tok
oc-dashboard-org-projyr = proj/yr
oc-dashboard-org-name = Organization
oc-dashboard-org-personal = You (billed)
oc-dashboard-org-fy-prefix = FY
oc-dashboard-org-asof = as of
oc-dashboard-section-by-model = By Model
oc-dashboard-section-by-agent = By Agent
oc-dashboard-section-command = Command
oc-dashboard-section-prompt = Prompt
oc-dashboard-section-last-output = Last Output
oc-dashboard-section-manual-run = Manual Run
oc-dashboard-section-recent-runs = Recent Runs
oc-dashboard-actions = Actions
oc-dashboard-cron-action-hint = { $run } run now · { $refresh } refresh
oc-dashboard-loading-runs = Loading runs…
oc-dashboard-runs-error = Failed to load runs
oc-dashboard-no-runs = No runs recorded yet
oc-dashboard-run-succeeded = Run succeeded
oc-dashboard-run-failed = Run failed
oc-dashboard-run-running = Run started: { $id }
oc-dashboard-run-already-running = Run already in progress: { $id }
oc-dashboard-run-pending-status = running
oc-dashboard-run-pending-window = starting now
oc-dashboard-run-pending-output = Manual trigger in progress…

oc-dashboard-yes = yes
oc-dashboard-no = no
oc-dashboard-enabled = enabled
oc-dashboard-disabled = disabled

oc-quickstart-title = Quickstart
oc-quickstart-selector-model-provider = Model provider
oc-quickstart-selector-risk-profile = Risk profile
oc-quickstart-selector-runtime-profile = Runtime profile
oc-quickstart-selector-memory = Memory
oc-quickstart-selector-channels = Channels (optional)
oc-quickstart-selector-peer-groups = Peer groups (optional)
oc-quickstart-selector-agent = Agent
oc-quickstart-selector-submit = Submit

oc-quickstart-reuse-alias-help = Reuse this alias instead of creating a new one.
oc-quickstart-existing-providers-heading = Existing providers
oc-quickstart-existing-providers-show-more = [+ Show { $count } more existing providers]
oc-quickstart-existing-providers-show-fewer = [- Show fewer existing providers]
oc-quickstart-new-provider-heading = Create new provider
oc-quickstart-provider-alias-exists = Alias `{ $alias }` already exists. Pick another alias or use the existing provider.

oc-quickstart-risk-locked-down = Locked Down
oc-quickstart-risk-locked-down-desc = Tight defaults. Workspace-only fs, approval on med/high risk.
oc-quickstart-risk-balanced = Balanced
oc-quickstart-risk-balanced-desc = Day-to-day defaults. Approval on risky ops. Recommended.
oc-quickstart-risk-yolo = YOLO
oc-quickstart-risk-yolo-desc = Full autonomy. No approval gates. Use on disposable machines only.

oc-quickstart-runtime-tight = Tight
oc-quickstart-runtime-tight-desc = Low ceilings on iterations and tokens.
oc-quickstart-runtime-local-small = Local Small
oc-quickstart-runtime-local-small-desc = Compact limits for smaller local models with structured tool calls.
oc-quickstart-runtime-balanced = Balanced
oc-quickstart-runtime-balanced-desc = Sensible ceilings. Recommended.
oc-quickstart-runtime-unbounded = Unbounded
oc-quickstart-runtime-unbounded-desc = No artificial caps.

oc-quickstart-provider-local = Local. No credential required.
oc-quickstart-provider-cloud = Cloud. Provide an API key when prompted.

oc-quickstart-submit-create = Create the agent


oc-quickstart-modal-action-move = move
oc-quickstart-modal-action-pick = pick
oc-quickstart-modal-action-cancel = cancel
oc-quickstart-modal-action-accept = accept
oc-quickstart-modal-action-pick-on-enum = pick on ‹enum›
oc-quickstart-modal-action-activate = activate
oc-quickstart-modal-action-delete = delete
oc-quickstart-modal-action-close = close
oc-quickstart-modal-action-edit-name = type to edit name
oc-quickstart-modal-action-on-file-rows = on file rows
oc-quickstart-modal-action-save = save
oc-quickstart-modal-type-prefix = Type:
oc-quickstart-field-label-alias = Alias
oc-quickstart-field-help-alias = Unique identifier for this provider instance. Change from "default" if your config already has one with that alias.
oc-quickstart-model-loading = Loading models for { $provider }...
oc-quickstart-model-retrying = Retrying model catalog for { $provider }...
oc-quickstart-model-catalog-empty = catalog unavailable - type a model name
oc-quickstart-action-done = Done
oc-quickstart-no-peer-groups = No peer groups configured. Optional — agents can still send messages to channels.

oc-quickstart-help-external-peers = Comma- or newline-separated. Blank = no external peers.

oc-quickstart-status-submitting = Submitting…
oc-quickstart-status-created = Created `{ $alias }`. Reloading daemon — Chat will open when reconnected…
oc-quickstart-status-errors = { $count } error(s) — fix selectors and resubmit
oc-quickstart-status-first-error = { $where }{ $field }: { $message }{ $more }
oc-quickstart-status-more-errors = { " " }(+{ $count } more)
oc-quickstart-status-can-create = All required selectors ✓. Press `{ $chord }` to Create.
oc-quickstart-status-hint = ↑/↓ to move, Enter to open. `{ $chord }` enables when all required selectors are ✓.
oc-quickstart-missing-model-provider = Choose a model provider before creating.
oc-quickstart-missing-risk-profile = Choose a risk profile before creating.
oc-quickstart-missing-runtime-profile = Choose a runtime profile before creating.
oc-quickstart-missing-memory = Choose a memory backend before creating.
oc-quickstart-missing-agent = Name the agent before creating.

oc-quickstart-channels-empty = No channels configured. An agent without channels still works via `omnesagent agent <name>` from the CLI.
oc-quickstart-channels-add = + Add channel
oc-quickstart-peers-add = + Add peer group
oc-quickstart-block-channels = Channels
oc-quickstart-block-peers = Peer groups
oc-quickstart-block-agent = Agent
oc-quickstart-personality-help = Personality files (e=edit, t=use template, c=clear)
oc-quickstart-save-and-close = Save & close
oc-quickstart-no-template = No template is available for `{ $filename }`
oc-quickstart-agent-name-field = name
oc-quickstart-file-bytes = { $bytes } bytes

oc-chat-pane-chat = Chat
oc-chat-pane-acp = ACP

oc-chat-no-agents = No enabled agents yet. Open Quickstart to create one, or use Config to add and enable an agent.
oc-chat-error-fetch-agents = Failed to fetch agents: { $error }
oc-chat-history-trimmed = Earlier conversation history was trimmed: { $reason } ({ $dropped } messages dropped; { $kept } turns kept).
oc-chat-error-create-session = Failed to create session: { $error }
oc-chat-session-switch-error = Failed to switch session: { $error }
oc-chat-elicitation-dropped = A prompt from the agent was dropped before it could be shown (the client fell behind). The agent's question may be waiting; try again.
oc-chat-session-restarted = New session started.
oc-chat-session-restart-error = Failed to start a new session: { $error }

oc-chat-thinking-visible = Thinking output: visible
oc-chat-thinking-hidden = Thinking output: hidden

# Model picker slash commands
oc-model-picker-hint = Type a model name after /model, or Tab to autocomplete.
oc-model-provider-picker-hint = Type a model_provider after /model-provider, then pick a model with /model.
oc-model-picker-title = Select model
oc-model-provider-picker-title = Select model_provider
oc-model-switch-applying = Applying model change…
oc-model-switch-model-ok = Model switched to { $model }.
oc-model-switch-provider-ok = model_provider switched to { $provider } (model: { $model }).
oc-model-switch-failed = Model switch failed: { $error }
oc-model-catalog-no-provider = Could not resolve this agent's model_provider from config.
oc-model-catalog-empty = No models available for the active model_provider.
oc-model-catalog-loading = Loading models…
oc-model-provider-catalog-failed = Could not load model_providers: { $error }

oc-chat-label-you = You:
oc-chat-label-agent = Agent:

oc-chat-loading-agents = Loading agents…
oc-chat-loading-agents-msg = Loading agents...
oc-chat-picker-header = Select an agent
oc-chat-picker-header-hint = ({ $keys })

oc-chat-help-navigate = Navigate
oc-chat-help-select-agent = Select agent
oc-chat-help-quit = Quit
oc-chat-help-switch-session = Switch session
oc-chat-help-close = Close
oc-chat-help-cancel = Cancel
oc-chat-help-approve = Approve
oc-chat-help-always-approve = Always approve
oc-chat-help-deny = Deny
oc-chat-help-cancel-turn = Cancel turn
oc-chat-help-move-up = Move cursor up
oc-chat-help-move-down = Move cursor down
oc-chat-help-extend-selection = Extend selection
oc-chat-help-yank-selection = Yank selection
oc-chat-help-return-to-input = Return to input
oc-chat-help-browse-mode = Browse mode
oc-chat-help-scroll-conversation = Scroll conversation
oc-chat-help-toggle-thoughts = Toggle thoughts
oc-chat-help-new-session = New session
oc-chat-session-list-resume-title = Saved sessions (Enter=resume, Esc=new)
oc-chat-session-list-switch-title = Sessions (Enter=switch, Esc=close)
oc-elicit-help-toggle = Toggle choice
oc-elicit-help-confirm = Confirm choice
oc-elicit-help-cancel = Cancel

oc-chat-approval-title = Approve tool call: { $tool }  [{ $secs }s]
oc-chat-approval-action-allow = Allow
oc-chat-approval-action-always = Always
oc-chat-approval-action-reject = Reject
oc-chat-approval-action-edit = Edit


oc-chat-clipboard-you = You: { $text }
oc-chat-clipboard-agent = Agent: { $text }
oc-chat-copied-clipboard = Copied to clipboard
oc-chat-copy-message = [Copy]
oc-chat-copy-message-copied = [Copied]
oc-chat-context-menu-copy = Copy
oc-chat-context-menu-send-now = Send now
oc-chat-context-menu-edit = Edit
oc-chat-context-menu-delete = Delete

oc-config-breadcrumb-root = Config
oc-config-section-detail-hint = { $open } or { $into } to open this section
oc-config-breadcrumb-new = New

oc-config-personality-over-limit = Over { $limit } char limit — cannot save
oc-config-alias-create-hint = Enter a name for the new alias
oc-config-personality-help-blurb = Personality files shape your agent's voice and context.
oc-config-skills-help-blurb = Skills in this bundle. { $enter_chord } to edit SKILL.md, { $archive_chord } to archive.

oc-config-field-type-prefix = Type:
oc-config-field-type-secret-suffix = (secret — input hidden)
oc-config-field-type-string-array-suffix = (one entry per line; { $newline_chord }=new line, { $save_chord }=save)

oc-config-help-navigate = Navigate
oc-config-help-switch-section = Switch config section
oc-config-help-open-section = Open section
oc-config-help-clear-filter = Clear filter
oc-config-help-this-help = This help
oc-config-help-filter = Filter
oc-config-help-quit = Quit
oc-config-help-mouse-label = Mouse
oc-config-help-mouse-open = Click, scroll, double-click to open
oc-config-help-mouse-tabs-edit = Click, scroll, click tabs, double-click to edit
oc-config-help-mouse-edit = Click, scroll, double-click to edit
oc-config-help-mouse-save = Click, scroll, double-click to save
oc-config-help-mouse-tabs = Click, scroll, click tabs
oc-config-help-open-type = Open type
oc-config-help-back = Back
oc-config-help-open-alias = Open alias
oc-config-help-delete-alias = Delete alias
oc-config-help-create-alias = Create alias
oc-config-help-cancel = Cancel
oc-config-help-edit-field = Edit field
oc-config-help-save = Save
oc-config-help-back-to-files = Back to files
oc-config-help-switch-tabs = Switch tabs
oc-config-help-edit-file = Edit file
oc-config-help-fill-from-template = Fill from template
oc-config-help-edit-skill = Edit skill
oc-config-help-archive-skill = Archive skill
oc-config-help-back-to-skills = Back to skills
oc-config-help-save-selection = Save selection
oc-config-help-new-line-entry = New line (new entry)
oc-config-help-save-array = Save array
oc-config-help-save-value = Save value
oc-config-help-reset-default = Reset to default

oc-config-status-alias-empty = Alias name cannot be empty
oc-config-status-alias-deleted = Deleted { $alias }
oc-config-status-alias-create-failed = Create failed: { $err }
oc-config-status-delete-failed = Delete failed: { $err }
oc-config-status-field-reset = Reset { $prop }
oc-config-status-load-failed = Load failed: { $err }
oc-config-status-save-failed = Save failed: { $err }
oc-config-status-personality-loading-file = Loading { $filename }...
oc-config-status-personality-saving-file = Saving { $filename }...
oc-config-status-personality-saved-file = Saved { $filename }
oc-config-status-template-loaded = Template loaded for { $filename }
oc-config-status-template-missing = No template available for { $filename }
oc-config-status-template-fetch-failed = Template fetch failed: { $err }
oc-config-status-skill-loading = Loading { $name }...
oc-config-status-skill-saving = Saving { $name }...
oc-config-status-skill-saved = Saved { $name }
oc-config-status-skill-deleting = Deleting { $name }...
oc-config-status-skill-archived = Archived { $name }
oc-config-status-fetching-models = Fetching models for { $family }...
oc-config-status-field-set = Set { $prop }
oc-config-status-invalid-float = Invalid value for { $prop }: expected a number
oc-config-status-invalid-integer = Invalid value for { $prop }: expected an integer
oc-config-status-set-failed = Set failed: { $err }
oc-config-status-loading-personality = Loading personality files...
oc-config-status-loading-skills = Loading skills...
oc-config-status-fetching-templates = Fetching templates...
oc-config-status-unsaved-discarded = Unsaved changes discarded
oc-config-status-no-models = No models returned — enter manually
oc-config-status-model-fetch-failed = Model fetch failed — enter manually
oc-config-status-loading-aliases = Loading options...
oc-config-status-no-aliases = None configured yet. Enter manually.
oc-config-status-alias-fetch-failed = Options fetch failed. Enter manually.

oc-config-footer-action-create = create
oc-config-footer-action-cancel = cancel
oc-config-footer-action-save = save
oc-config-footer-action-edit = edit
oc-config-footer-action-reset = reset
oc-config-footer-action-clear-filter = clear filter
oc-config-footer-action-back-to-files = back to files
oc-config-footer-action-back-to-skills = back to skills
oc-config-footer-action-help = help
oc-config-footer-action-new-line = new line

## Inline hint shown on the selected config field row. The { $keys } placeholder
## is resolved from the current keybinding for ConfigTabAction::Enter.
oc-config-field-edit-hint = { $keys } → press to edit

oc-doctor-log-path = log: { $path }
