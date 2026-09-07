// Desktop Inspector Panel: Run Timeline, Live Computer Use, and SOP Approvals.

import 'package:flutter/material.dart';
import 'package:font_awesome_flutter/font_awesome_flutter.dart';
import '../../theme/desktop_theme.dart';
import '../workspace/task_workspace_controller.dart';

class DesktopInspectorPanel extends StatefulWidget {
  final DesktopTaskWorkspaceController controller;

  const DesktopInspectorPanel({super.key, required this.controller});

  @override
  State<DesktopInspectorPanel> createState() => _DesktopInspectorPanelState();
}

class _DesktopInspectorPanelState extends State<DesktopInspectorPanel>
    with SingleTickerProviderStateMixin {
  late TabController tabController;

  @override
  void initState() {
    super.initState();
    tabController = TabController(length: 3, vsync: this);
  }

  @override
  void dispose() {
    tabController.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    return Container(
      width: 340,
      decoration: const BoxDecoration(
        color: DesktopTheme.bgSidebar,
        border: Border(
          left: BorderSide(color: DesktopTheme.borderSubtle, width: 1),
        ),
      ),
      child: Column(
        children: [
          // Inspector TabBar
          Container(
            height: 44,
            decoration: const BoxDecoration(
              color: DesktopTheme.bgSurface,
              border: Border(
                bottom: BorderSide(color: DesktopTheme.borderSubtle, width: 1),
              ),
            ),
            child: TabBar(
              controller: tabController,
              labelColor: DesktopTheme.accentSky,
              unselectedLabelColor: DesktopTheme.textMuted,
              indicatorColor: DesktopTheme.accentSky,
              indicatorSize: TabBarIndicatorSize.tab,
              labelStyle: const TextStyle(fontSize: 12, fontWeight: FontWeight.bold),
              tabs: const [
                Tab(text: 'Timeline'),
                Tab(text: 'Computer Use'),
                Tab(text: 'Approvals'),
              ],
            ),
          ),

          // Tab views
          Expanded(
            child: TabBarView(
              controller: tabController,
              children: [
                _buildTimelineTab(),
                _buildComputerUseTab(),
                _buildApprovalsTab(),
              ],
            ),
          ),
        ],
      ),
    );
  }

  Widget _buildTimelineTab() {
    return ListView.builder(
      padding: const EdgeInsets.all(12),
      itemCount: widget.controller.runTimelineSteps.length,
      itemBuilder: (context, index) {
        final step = widget.controller.runTimelineSteps[index];
        return Container(
          margin: const EdgeInsets.only(bottom: 8),
          padding: const EdgeInsets.all(10),
          decoration: BoxDecoration(
            color: DesktopTheme.bgSurface,
            borderRadius: BorderRadius.circular(6),
            border: Border.all(color: DesktopTheme.borderSubtle),
          ),
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              Row(
                children: [
                  Container(
                    width: 6,
                    height: 6,
                    decoration: const BoxDecoration(
                      shape: BoxShape.circle,
                      color: DesktopTheme.statusSuccess,
                    ),
                  ),
                  const SizedBox(width: 8),
                  Expanded(
                    child: Text(
                      step['title'] ?? '',
                      style: const TextStyle(
                        fontSize: 12,
                        fontWeight: FontWeight.w600,
                        color: DesktopTheme.textPrimary,
                      ),
                    ),
                  ),
                  Text(
                    step['duration'] ?? '',
                    style: const TextStyle(
                      fontSize: 11,
                      fontFamily: 'Consolas',
                      color: DesktopTheme.textMuted,
                    ),
                  ),
                ],
              ),
              const SizedBox(height: 4),
              Text(
                'tool: ${step['tool']}',
                style: const TextStyle(
                  fontSize: 11,
                  fontFamily: 'Consolas',
                  color: DesktopTheme.accentCyan,
                ),
              ),
            ],
          ),
        );
      },
    );
  }

  Widget _buildComputerUseTab() {
    return Padding(
      padding: const EdgeInsets.all(14),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Row(
            children: [
              const Icon(
                FontAwesomeIcons.desktop,
                size: 13,
                color: DesktopTheme.accentSky,
              ),
              const SizedBox(width: 8),
              const Text(
                'Live Screen / Browser View',
                style: TextStyle(
                  fontSize: 12,
                  fontWeight: FontWeight.bold,
                  color: DesktopTheme.textPrimary,
                ),
              ),
              const Spacer(),
              Container(
                padding: const EdgeInsets.symmetric(horizontal: 6, vertical: 2),
                decoration: BoxDecoration(
                  color: DesktopTheme.statusSuccess.withOpacity(0.12),
                  borderRadius: BorderRadius.circular(4),
                ),
                child: const Text(
                  'READY',
                  style: TextStyle(
                    fontSize: 10,
                    fontWeight: FontWeight.bold,
                    color: DesktopTheme.statusSuccess,
                  ),
                ),
              ),
            ],
          ),
          const SizedBox(height: 12),
          Expanded(
            child: Container(
              width: double.infinity,
              decoration: BoxDecoration(
                color: DesktopTheme.bgCanvas,
                borderRadius: BorderRadius.circular(8),
                border: Border.all(color: DesktopTheme.borderSubtle),
              ),
              child: const Column(
                mainAxisAlignment: MainAxisAlignment.center,
                children: [
                  Icon(
                    FontAwesomeIcons.display,
                    size: 36,
                    color: DesktopTheme.textMuted,
                  ),
                  SizedBox(height: 12),
                  Text(
                    'No active Computer Use session',
                    style: TextStyle(fontSize: 12, color: DesktopTheme.textMuted),
                  ),
                  SizedBox(height: 4),
                  Text(
                    'Subagent actions will stream live here',
                    style: TextStyle(fontSize: 11, color: DesktopTheme.textMuted),
                  ),
                ],
              ),
            ),
          ),
        ],
      ),
    );
  }

  Widget _buildApprovalsTab() {
    return Padding(
      padding: const EdgeInsets.all(14),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          const Text(
            'SOP & Security Approvals',
            style: TextStyle(
              fontSize: 12,
              fontWeight: FontWeight.bold,
              color: DesktopTheme.textPrimary,
            ),
          ),
          const SizedBox(height: 8),
          const Text(
            'Autonomous execution pauses and requires confirmation for risky commands.',
            style: TextStyle(fontSize: 11, color: DesktopTheme.textMuted),
          ),
          const SizedBox(height: 16),
          Expanded(
            child: Center(
              child: Column(
                mainAxisAlignment: MainAxisAlignment.center,
                children: [
                  Icon(
                    Icons.verified_user_outlined,
                    size: 36,
                    color: DesktopTheme.statusSuccess.withOpacity(0.7),
                  ),
                  const SizedBox(height: 10),
                  const Text(
                    'No pending approval requests',
                    style: TextStyle(
                      fontSize: 13,
                      fontWeight: FontWeight.w600,
                      color: DesktopTheme.textSecondary,
                    ),
                  ),
                  const SizedBox(height: 4),
                  const Text(
                    'All safe tools executing normally',
                    style: TextStyle(fontSize: 11, color: DesktopTheme.textMuted),
                  ),
                ],
              ),
            ),
          ),
        ],
      ),
    );
  }
}
