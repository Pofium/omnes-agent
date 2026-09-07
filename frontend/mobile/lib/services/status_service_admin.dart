import 'package:flutter/material.dart';

import '../helper/local_storage.dart';

class StatusService {
  static init() {
    debugPrint('--------StatusService activate-------------');

    LocalStorage.saveChatStatus(chatStatusBool: true);
    LocalStorage.saveImageStatus(imageStatusBool: true);
    LocalStorage.saveContentStatus(value: true);
    LocalStorage.saveKeyWordStatus(value: true);
    LocalStorage.saveDietChartStatus(value: true);
    LocalStorage.saveCoverLetterStatus(value: true);
    LocalStorage.saveRandomLoveStatus(value: true);
    LocalStorage.saveScheduleStatus(value: true);

    LocalStorage.saveSubscriptionStatus(subscriptionStatusBool: true);

    LocalStorage.saveStripeStatus(stripeStatusBool: true);
    LocalStorage.savePaypalStatus(paypalStatusBool: true);
    LocalStorage.savesslStatus(value: true);
    LocalStorage.savePayStatus(value: true);
    LocalStorage.savePayStackStatus(value: true);
    LocalStorage.savePayStackCardStatus(value: true);
    LocalStorage.savePayStackBankStatus(value: true);
    LocalStorage.saveFlutterWaveStatus(value: true);

    LocalStorage.saveAllAdStatus(value: false);
    LocalStorage.saveUnityAdStatus(value: false);
    LocalStorage.saveAdMobAdStatus(value: false);
  }
}