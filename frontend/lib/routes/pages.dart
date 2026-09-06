import 'package:get/get.dart';
import '../binding/splash_binding.dart';
import '../features/auth/login_screen.dart';
import '../features/automation/cron_screen.dart';
import '../features/chat/chat_screen.dart';
import '../features/files/pdf_view_screen.dart';
import '../features/home/home_screen.dart';
import '../features/memory/memory_screen.dart';
import '../features/profile/update_profile_screen.dart';
import '../features/projects/projects_screen.dart';
import '../features/sessions/sessions_screen.dart';
import '../features/settings/settings_screen.dart';
import '../features/splash/splash_screen.dart';
import '../features/stats/stats_screen.dart';
import '../features/doctor/doctor_screen.dart';
import '../features/integrations/integrations_screen.dart';
import '../features/logs/logs_screen.dart';
import '../features/pairing/pairing_screen.dart';
import '../features/skills/skills_screen.dart';
import '../features/tools/tools_screen.dart';
import '../utils/strings.dart';
import '../widgets/others/webview_widget.dart';
import 'routes.dart';

class Pages {
  static var list = [
    GetPage(
      name: Routes.splashScreen,
      page: () => const SplashScreen(),
      binding: SplashBinding(),
    ),
    GetPage(
      name: Routes.loginScreen,
      page: () => LogInScreen(),
    ),
    GetPage(
      name: Routes.homeScreen,
      page: () => const HomeScreen(),
    ),
    GetPage(
      name: Routes.chatScreen,
      page: () => ChatScreen(),
    ),
    GetPage(
      name: Routes.projectsScreen,
      page: () => const ProjectsScreen(),
    ),
    GetPage(
      name: Routes.sessionsScreen,
      page: () => const SessionsScreen(),
    ),
    GetPage(
      name: Routes.memoryScreen,
      page: () => const MemoryScreen(),
    ),
    GetPage(
      name: Routes.cronScreen,
      page: () => const CronScreen(),
    ),
    GetPage(
      name: Routes.statsScreen,
      page: () => const StatsScreen(),
    ),
    GetPage(
      name: Routes.settingsScreen,
      page: () => SettingsScreen(),
    ),
    GetPage(
      name: Routes.updateProfileScreen,
      page: () => UpdateProfileScreen(),
    ),
    GetPage(
      name: Routes.toolsScreen,
      page: () => const ToolsScreen(),
    ),
    GetPage(
      name: Routes.skillsScreen,
      page: () => const SkillsScreen(),
    ),
    GetPage(
      name: Routes.doctorScreen,
      page: () => const DoctorScreen(),
    ),
    GetPage(
      name: Routes.logsScreen,
      page: () => const LogsScreen(),
    ),
    GetPage(
      name: Routes.integrationsScreen,
      page: () => const IntegrationsScreen(),
    ),
    GetPage(
      name: Routes.pairingScreen,
      page: () => const PairingScreen(),
    ),
    GetPage(
      name: Routes.pdfViewerWidget,
      page: () {
        final args = Get.arguments;
        final name = (args is Map && args['name'] != null) ? args['name'].toString() : 'Document.pdf';
        final pdfBytes = (args is Map && args['pdfBytes'] is List<int>) ? args['pdfBytes'] as List<int> : const <int>[];
        return PdfViewScreen(name: name, pdfBytes: pdfBytes);
      },
    ),
    GetPage(
      name: Routes.privacyPolicy,
      page: () => const WebviewWidget(
        mainUrl: Strings.privacyPolicyUrl,
        appBarTitle: Strings.privacyPolicy,
      ),
    ),
    GetPage(
      name: Routes.termsAndCondition,
      page: () => const WebviewWidget(
        mainUrl: Strings.termsUrl,
        appBarTitle: Strings.terms,
      ),
    ),
    GetPage(
      name: Routes.refundPolicy,
      page: () => const WebviewWidget(
        mainUrl: Strings.refundPolicyUrl,
        appBarTitle: Strings.refundPolicy,
      ),
    ),
  ];
}
