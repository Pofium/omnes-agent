class ApiConfig {
  static const String stripeUrl = "https://api.stripe.com/v1/payment_intents";
  static const String stripeSecretKey = String.fromEnvironment("STRIPE_SECRET_KEY", defaultValue: "");
  static const String stripePublishableKey = String.fromEnvironment("STRIPE_PUBLISHABLE_KEY", defaultValue: "");

  static String payStackPublicKey = const String.fromEnvironment("PAYSTACK_PUBLIC_KEY", defaultValue: "");
  static String payStackServerKey = const String.fromEnvironment("PAYSTACK_SERVER_KEY", defaultValue: "");

  static String flutterwavePublicKey = const String.fromEnvironment("FLUTTERWAVE_PUBLIC_KEY", defaultValue: "");

  static const double premiumSubscriptionFee = 19.0;

  static const int freeMessageLimit = 2;
  static const int freeImageGenLimit = 1;
  static const int freeContentLimit = 2;
  static const int freeHashTagLimit = 2;

  static const int premiumMessageLimit = 100;
  static const int premiumImageGenLimit = 50;
  static const int premiumContentLimit = 100;
  static const int premiumHashTagLimit = 100;
  static const int premiumDuration = 30;
}
