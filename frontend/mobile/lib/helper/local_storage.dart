import 'package:flutter/widgets.dart';
import 'package:get/get.dart';
import 'package:get_storage/get_storage.dart';

import '../model/notification_model/notification_model.dart';
import '../model/schedule/schedule_model.dart';
import 'set_schedule_local_notification.dart';

const String idKey = "idKey";

const String nameKey = "nameKey";

const String tokenKey = "tokenKey";

const String emailKey = "emailKey";

const String imageKey = "imageKey";

const String isFreeUserToken = "isFreeUser";
const String chatGptApiKey = "chatGptApiKey";
const String paypalClientId = "paypalClientId";
const String stripeClientId = "stripeClientId";
const String paypalSecret = "paypalSecret";
const String textCount = "textCount";
const String imageCount = "imageCount";
const String contentCount = "contentCount";
const String hashTagCount = "hashTagCount";
const String date = "date";

const String subscriptionDate = "subscriptionDate";

const String anouncment = "anouncment";

const String chatStatus = "chatStatus";
const String imageStatus = "imageStatus";
const String contentStatus = "contentStatus";
const String keyWordStatus = "keyWordStatus";
const String dietChartStatus = "dietChartStatus";
const String coverLetterStatus = "coverLetterStatus";
const String randomLoveStatus = "randomLoveStatus";
const String scheduleStatus = "scheduleStatus";
const String subscriptionStatus = "subscriptionStatus";
const String paypalStatus = "paypalStatus";
const String stripeStatus = "stripeStatus";
const String sslStatus = "sslStatus";
const String payStatus = "payStatus";
const String paystackStatus = "paystackStatus";
const String payStackCardStatus = "payStackCardStatus";
const String payStackBankStatus = "payStackBankStatus";
const String allAdStatus = "allAdStatus";
const String unityAdStatus = "unityAdStatus";
const String adMobAdStatus = "adMobAdStatus";
const String flutterWaveStatus = "flutterWaveStatus";

const String isLoggedInKey = "isLoggedInKey";

const String isFreeUserKey = "isFreeUserKey";

const String isDataLoadedKey = "isDataLoadedKey";

const String isOnBoardDoneKey = "isOnBoardDoneKey";

const String isScheduleEmptyKey = "isScheduleEmptyKey";

const String language = "language";
const String smallLanguage = "smallLanguage";
const String capitalLanguage = "capitalLanguage";
const String themeName = "themeName";

const String selectedToken = "selectedToken";
const String selectedModel = "selectedModel";
const String selectedImageType = "selectedImageType";

const String scheduleListKey = "scheduleListKey";

class LocalStorage {
  static Future<void> saveLanguage({
    required String langSmall,
    required String langCap,
    required String languageName,
  }) async {
    final box1 = GetStorage();
    final box2 = GetStorage();
    final box3 = GetStorage();

    var locale = Locale(langSmall, langCap);
    Get.updateLocale(locale);
    await box1.write(smallLanguage, langSmall);
    await box2.write(capitalLanguage, langCap);
    await box3.write(language, languageName);
  }

  static List getLanguage() {
    String small = GetStorage().read(smallLanguage) ?? 'en';
    String capital = GetStorage().read(capitalLanguage) ?? 'US';
    String languages = GetStorage().read(language) ?? 'English';
    return [small, capital, languages];
  }

  /// ------------------------ schedule locla manage
  static List<ScheduleModel> getScheduleModels(){
    List data = GetStorage().read(scheduleListKey) ?? [];
    List<ScheduleModel> schedules = data.map((map) => ScheduleModel.fromJson(map)).toList();
    return schedules;
  }

  static Future<void> saveScheduleModels(ScheduleModel schedule)async{
    List data = GetStorage().read(scheduleListKey) ?? [];
    List schedules = data;
    schedules.add(schedule.toJson());

    final box = GetStorage();
    await box.write(scheduleListKey, schedules);

    /// todo - add schedule local notification
    await SetScheduleNotification.setSchedule(schedule);
  }

  static Future<void> getDailyScheduleModels()async{
    List data = GetStorage().read(scheduleListKey) ?? [];
    List schedules = data;

    final box = GetStorage();
    await box.write(scheduleListKey, schedules);

    for (var element in schedules) {
      if(element.scheduleType == "Daily") {
        await SetScheduleNotification.setSchedule(element);
      }
    }

  }

  static Future<void> removeASchedule(int index)async{
    List data = GetStorage().read(scheduleListKey) ?? [];
    List schedules = data;
    schedules.removeAt(index);

    final box = GetStorage();
    await box.write(scheduleListKey, schedules);

    /// todo - remove a schedule local notification

    await SetScheduleNotification.removeIDNotification(index);
  }

  static Future<void> deleteAllSchedule()async{
    final box = GetStorage();
    await box.remove(scheduleListKey);

    await SetScheduleNotification.cancelAllNotifications();
  }

  /// ----- end schedule  =---------------------



  /// ----- save theme
  static Future<void> saveTheme({
    required int themeStateName,
  }) async {
    final box1 = GetStorage();
    await box1.write(themeName, themeStateName);
  }

  static getThemeState() {
    return GetStorage().read(themeName) ?? 0;
  }

  static Future<void> saveId({required String id}) async {
    final box = GetStorage();

    await box.write(idKey, id);
  }

  static Future<void> saveName({required String name}) async {
    final box = GetStorage();

    await box.write(nameKey, name);
  }

  static Future<void> saveEmail({required String email}) async {
    final box = GetStorage();

    await box.write(emailKey, email);
  }

  static Future<void> saveToken({required String token}) async {
    final box = GetStorage();

    await box.write(tokenKey, token);
  }

  static Future<void> saveImage({required String image}) async {
    final box = GetStorage();

    await box.write(imageKey, image);
  }

  static Future<void> isLoginSuccess({required bool isLoggedIn}) async {
    final box = GetStorage();

    await box.write(isLoggedInKey, isLoggedIn);
  }

  static Future<void> saveChatStatus({required bool chatStatusBool}) async {
    final box = GetStorage();

    await box.write(chatStatus, chatStatusBool);
  }

  static Future<void> saveImageStatus({required bool imageStatusBool}) async {
    final box = GetStorage();

    await box.write(imageStatus, imageStatusBool);
  }

  static Future<void> saveContentStatus({required bool value}) async {
    final box = GetStorage();

    await box.write(contentStatus, value);
  }

  static Future<void> saveKeyWordStatus({required bool value}) async {
    final box = GetStorage();

    await box.write(keyWordStatus, value);
  }

  static Future<void> saveDietChartStatus({required bool value}) async {
    final box = GetStorage();

    await box.write(dietChartStatus, value);
  }

  static Future<void> saveCoverLetterStatus({required bool value}) async {
    final box = GetStorage();

    await box.write(coverLetterStatus, value);
  }

  static Future<void> saveRandomLoveStatus({required bool value}) async {
    final box = GetStorage();

    await box.write(randomLoveStatus, value);
  }

  static Future<void> saveScheduleStatus({required bool value}) async {
    final box = GetStorage();

    await box.write(scheduleStatus, value);
  }

  static Future<void> saveSubscriptionStatus(
      {required bool subscriptionStatusBool}) async {
    final box = GetStorage();

    await box.write(subscriptionStatus, subscriptionStatusBool);
  }

  static Future<void> saveStripeStatus({required bool stripeStatusBool}) async {
    final box = GetStorage();

    await box.write(stripeStatus, stripeStatusBool);
  }

  static Future<void> savePaypalStatus({required bool paypalStatusBool}) async {
    final box = GetStorage();

    await box.write(paypalStatus, paypalStatusBool);
  }

  static Future<void> savesslStatus({required bool value}) async {
    final box = GetStorage();

    await box.write(sslStatus, value);
  }

  static Future<void> savePayStatus({required bool value}) async {
    final box = GetStorage();

    await box.write(payStatus, value);
  }


  static Future<void> saveFlutterWaveStatus({required bool value}) async {
    final box = GetStorage();

    await box.write(flutterWaveStatus, value);
  }

  static Future<void> savePayStackStatus({required bool value}) async {
    final box = GetStorage();

    await box.write(paystackStatus, value);
  }

  static Future<void> savePayStackCardStatus({required bool value}) async {
    final box = GetStorage();

    await box.write(payStackCardStatus, value);
  }

  static Future<void> savePayStackBankStatus({required bool value}) async {
    final box = GetStorage();

    await box.write(payStackBankStatus, value);
  }


  static Future<void> saveUnityAdStatus({required bool value}) async {
    final box = GetStorage();

    await box.write(unityAdStatus, value);
  }

  static Future<void> saveAdMobAdStatus({required bool value}) async {
    final box = GetStorage();

    await box.write(adMobAdStatus, value);
  }

  static Future<void> saveAllAdStatus({required bool value}) async {
    final box = GetStorage();

    await box.write(allAdStatus, value);
  }

  static Future<void> dataLoaded({required bool isDataLoad}) async {
    final box = GetStorage();

    await box.write(isDataLoadedKey, isDataLoad);
  }

  static Future<void> scheduleEmpty({required bool isScheduleEmpty}) async {
    final box = GetStorage();

    await box.write(isScheduleEmptyKey, isScheduleEmpty);
  }

  static Future<void> showIsFreeUser({required bool isShowAdYes}) async {
    final box = GetStorage();

    await box.write(isFreeUserToken, isShowAdYes);
  }

  static Future<void> saveChatGptApiKey({required String key}) async {
    final box = GetStorage();
    await box.write(chatGptApiKey, key);
  }

  static Future<void> savePaypalClientId({required String key}) async {
    final box = GetStorage();
    await box.write(paypalClientId, key);
  }

  static Future<void> saveStripeClientId({required String key}) async {
    final box = GetStorage();
    await box.write(stripeClientId, key);
  }

  static Future<void> savePaypalSecret({required String key}) async {
    final box = GetStorage();
    await box.write(paypalSecret, key);
  }

  static Future<void> saveTextCount({required int count}) async {
    final box = GetStorage();
    await box.write(textCount, count);
  }

  static Future<void> saveImageCount({required int count}) async {
    final box = GetStorage();
    await box.write(imageCount, count);
  }

  static Future<void> saveContentCount({required int count}) async {
    final box = GetStorage();
    await box.write(contentCount, count);
  }

  static Future<void> saveHashTagCount({required int count}) async {
    final box = GetStorage();
    await box.write(hashTagCount, count);
  }

  static Future<void> saveDate({required int value}) async {
    final box = GetStorage();
    await box.write(date, value);
  }

  static Future<void> saveSubscriptionDate({required DateTime date}) async {
    final box = GetStorage();
    await box.write(subscriptionDate, date);
  }

  static Future<void> saveAnouncment(
      {required NotificationModel notificationData}) async {
    final box = GetStorage();
    debugPrint(notificationData.toJson().toString());
    List<NotificationModel> list = GetStorage().read(anouncment) ?? [];
    list.add(notificationData);
    debugPrint(list.toString());
    debugPrint(list.toString());
    await box.write(anouncment, list);
  }

  static Future<void> saveOnboardDoneOrNot(
      {required bool isOnBoardDone}) async {
    final box = GetStorage();

    await box.write(isOnBoardDoneKey, isOnBoardDone);
  }

  static String? getId() {
    return GetStorage().read(idKey);
  }

  static String? getName() {
    return GetStorage().read(nameKey);
  }

  static String? getChatGptApiKey() {
    return GetStorage().read(chatGptApiKey);
  }

  static String? getPaypalClientId() {
    return GetStorage().read(paypalClientId);
  }

  static String? getStripeClientId() {
    return GetStorage().read(stripeClientId);
  }

  static String? getPaypalSecret() {
    return GetStorage().read(paypalSecret);
  }

  static int getTextCount() {
    return GetStorage().read(textCount) ?? 0;
  }

  static int getImageCount() {
    return GetStorage().read(imageCount) ?? 0;
  }

  static int getContentCount() {
    return GetStorage().read(contentCount) ?? 0;
  }

  static int getHashTagCount() {
    return GetStorage().read(hashTagCount) ?? 0;
  }

  static String getDateString() {
    int dateIntFormat = GetStorage().read(date) ?? 0;
    DateTime dateValue =
    DateTime.fromMillisecondsSinceEpoch(dateIntFormat, isUtc: true);
    String day = dateValue.day.toString();
    String month = dateValue.month.toString();
    String year = dateValue.year.toString();
    return "$year-$month-$day";
  }

  static int getDate() {
    return GetStorage().read(date) ?? 0;
  }

  static List<NotificationModel> getNotification() {
    return GetStorage().read(anouncment) ?? <NotificationModel>[];
  }

  static String? getEmail() {
    return GetStorage().read(emailKey);
  }

  static String? getToken() {
    var rtrn = GetStorage().read(tokenKey);

    debugPrint(rtrn == null ? "##Token is null###" : "");

    return rtrn;
  }

  static String? getImage() {
    return GetStorage().read(imageKey);
  }

  static bool isLoggedIn() {
    return GetStorage().read(isLoggedInKey) ?? false;
  }

  static bool getChatStatus() {
    return GetStorage().read(chatStatus) ?? true;
  }

  static bool getImageStatus() {
    return GetStorage().read(imageStatus) ?? true;
  }

  static bool getContentStatus() {
    return GetStorage().read(contentStatus) ?? true;
  }

  static bool getKeyWordStatus() {
    return GetStorage().read(keyWordStatus) ?? true;
  }

  static bool getDietChartStatus() {
    return GetStorage().read(dietChartStatus) ?? true;
  }

  static bool getCoverLetterStatus() {
    return GetStorage().read(coverLetterStatus) ?? true;
  }

  static bool getRandomLoveStatus() {
    return GetStorage().read(randomLoveStatus) ?? true;
  }

  static bool getScheduleStatus() {
    return GetStorage().read(scheduleStatus) ?? true;
  }

  static bool getSubscriptionStatus() {
    return GetStorage().read(subscriptionStatus) ?? true;
  }


  static bool getStripeStatus() {
    return GetStorage().read(stripeStatus) ?? true;
  }

  static bool getPaypalStatus() {
    return GetStorage().read(paypalStatus) ?? true;
  }

  static bool getSSLStatus() {
    return GetStorage().read(sslStatus) ?? true;
  }

  static bool getPayStatus() {
    return GetStorage().read(payStatus) ?? true;
  }

  static bool getFlutterWaveStatus() {
    return GetStorage().read(flutterWaveStatus) ?? true;
  }



  static bool getPayStackStatus() {
    return GetStorage().read(paystackStatus) ?? true;
  }

  static bool getPayStackCardStatus() {
    return GetStorage().read(payStackCardStatus) ?? true;
  }

  static bool getPayStackBankStatus() {
    return GetStorage().read(payStackBankStatus) ?? true;
  }

  static bool getUnityAdStatus() {
    return GetStorage().read(unityAdStatus) ?? true;
  }
  static bool getAdMobAdStatus() {
    return GetStorage().read(adMobAdStatus) ?? true;
  }
  static bool getAllAdStatus() {
    return GetStorage().read(allAdStatus) ?? true;
  }

  static bool isDataloaded() {
    return GetStorage().read(isDataLoadedKey) ?? false;
  }

  static bool isScheduleEmpty() {
    return GetStorage().read(isScheduleEmptyKey) ?? false;
  }

  static bool isOnBoardDone() {
    return GetStorage().read(isOnBoardDoneKey) ?? false;
  }

  static bool isFreeUser() {
    return GetStorage().read(isFreeUserToken) ?? true;
  }

  static String? get() {
    return GetStorage().read(nameKey);
  }

  static DateTime? getSubscriptionDate() {
    return GetStorage().read(subscriptionDate);
  }

  static Future<void> logout() async {
    final box = GetStorage();

    await box.remove(idKey);

    await box.remove(nameKey);

    await box.remove(emailKey);

    await box.remove(imageKey);

    await box.remove(isLoggedInKey);

    await box.remove(isOnBoardDoneKey);

    await box.remove(isFreeUserToken);

    await box.remove(date);
    await box.remove(imageCount);
    await box.remove(contentCount);
    await box.remove(hashTagCount);
    await box.remove(textCount);
    await box.remove(isFreeUserKey);

    await box.remove(isScheduleEmptyKey);

    await box.remove(subscriptionDate);
  }

  static int getSelectedToken() {
    return GetStorage().read(selectedToken) ?? 2000;
  }

  static Future<void> saveSelectedToken({required int value}) async {
    final box = GetStorage();

    await box.write(selectedToken, value);
  }

  static String getSelectedModel() {
    return GetStorage().read(selectedModel) ?? 'gpt-3.5-turbo';
  }

  static Future<void> saveSelectedModel({required String value}) async {
    final box = GetStorage();

    await box.write(selectedModel, value);
  }

  static String getSelectedImageType() {
    return GetStorage().read(selectedImageType) ?? '256x256';
  }

  static Future<void> saveSelectedImageType({required String value}) async {
    final box = GetStorage();

    await box.write(selectedImageType, value);
  }

  static const String autoTtsKey = "autoTtsKey";

  static bool getAutoTts() {
    return GetStorage().read(autoTtsKey) ?? false;
  }

  static Future<void> saveAutoTts({required bool value}) async {
    final box = GetStorage();
    await box.write(autoTtsKey, value);
  }

  static const String pinProtectionKey = "pinProtectionKey";
  static const String pinCodeKey = "pinCodeKey";
  static const String biometricAuthKey = "biometricAuthKey";
  static const String customProvidersKey = "customProvidersKey";

  static bool isPinProtectionEnabled() {
    final enabled = GetStorage().read(pinProtectionKey) ?? false;
    final pin = getPinCode();
    return enabled && pin.isNotEmpty;
  }

  static Future<void> savePinProtection({required bool value}) async {
    final box = GetStorage();
    await box.write(pinProtectionKey, value);
  }

  static String getPinCode() {
    return GetStorage().read(pinCodeKey) ?? '';
  }

  static Future<void> savePinCode({required String pin}) async {
    final box = GetStorage();
    await box.write(pinCodeKey, pin);
  }

  static bool isBiometricAuthEnabled() {
    return GetStorage().read(biometricAuthKey) ?? false;
  }

  static Future<void> saveBiometricAuth({required bool value}) async {
    final box = GetStorage();
    await box.write(biometricAuthKey, value);
  }

  static List<Map<String, dynamic>> getCustomProviders() {
    final raw = GetStorage().read(customProvidersKey);
    if (raw is List) {
      return raw.map((e) => Map<String, dynamic>.from(e as Map)).toList();
    }
    return [];
  }

  static Future<void> saveCustomProviders(List<Map<String, dynamic>> list) async {
    final box = GetStorage();
    await box.write(customProvidersKey, list);
  }
}

