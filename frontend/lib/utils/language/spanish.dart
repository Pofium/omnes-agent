import 'package:omagent_front/utils/config.dart';

import '../strings.dart';

class Spanish {
  static const String appName = Strings.appName;

  static const String chatWithOmAgent = "Chatear con $appName";
  static const String chatWithOmAgentSubTitle =
      "Chatear con $appName puede ser una forma divertida e interactiva de explorar el lenguaje y aprender cosas nuevas.";
  static const String settings = "Ajustes";
  static const String token = "Simbólico";
  static const String cancel = "Cancelar";
  static const String save = "Salvar";
  static const String skip = "Saltar";
  static const String subscriptionPlan = "Plan de Suscripción";
  static const String freeSubscription = "Suscripción gratuita";
  static const String notIncluded = "No incluido";
  static const String limitedChatting = "Chat limitado con $appName";
  static const String limitedCImage = "Generar imagen limitada";
  static String basicChatting =
      "${ApiConfig.freeMessageLimit.toString()} Chat ilimitado con $appName";
  static String basicImage =
      "${ApiConfig.freeImageGenLimit.toString()} Generación ilimitada de imágenes";

  static String unlimitedChatting =
      "${ApiConfig.premiumMessageLimit.toString()} Chat ilimitado con $appName";
  static String unlimitedImage =
      "${ApiConfig.premiumImageGenLimit.toString()} Generación ilimitada de imágenes";
  static const String freeSupport = "Soporte gratuito";
  static const String premiumSubscription = "Suscripción prémium";
  static const String google = "Inicia sesión con Google";
  static const String continueAsGuest = "Continuar como invitado";
  static const String searchSomething = "buscar algo";

  static const String generateAnyImage = "Generar imagen";
  static const String generateAnyImageSubTitle =
      "La generación de imágenes en IA se refiere al proceso de creación de nuevas imágenes utilizando algoritmos de inteligencia artificial.";
  static const String typeYour = "Escribe tu mensaje...";

  static const String writeSomething = "Escribe algo";
  static const String perMonth = "Por\nmes";

  static const String regenerateResponse = "Respuesta regenerada";
  static const String regeneratingResponse = "Respuesta regeneradora..";
  static const String clearConversation = "Conversación clara";
  static const String hello = "Hola";
  static const String helloGuest = "Hola invitada";

  static const String english = "Inglés";
  static const String arabic = "Arábica";
  static const String spanish = "Español";

  static const String download = "Descargar";
  static const String share = "Cuota";

  static const String facebook = "Sign In with Facebook";
  static const String youCanSend = "Puedes enviar";
  static const String messagesToTheBot = "Mensajes al bot";
  static const String messagesLeft = "Mensajes a la izquierda";
  static const String outOffLimit = "Lo siento, estás fuera del límite.";

  static const String enterYourName = "Introduzca su nombre";
  static const String enterYourEmail = "Introduce tu correo electrónico";
  static const String describeYourIssue = "Describe tu problema";
  static const String faceAnyProblem = "¿Se enfrenta a algún problema?";

  static const String emptyAnnouncement = "¡Anuncio vacío!";

  static const String logOut = "Cerrar sesión";
  static const String supportAndFeedback = "Soporte y Comentarios";
  static const String deleteAccount = "Borrar cuenta";
  static const String deleteYourAccount = "¿Quieres eliminar tu cuenta?";
  static const String alert = "¡Alerta!";

  static const String purchasePlan = "Plan de compra";
  static const String privacyPolicy = "Política de privacidad";
  static const String terms = "Términos y condiciones";
  static const String refundPolicy = "Política de reembolso";

  ///
  static const String chatStatus = "Estado de chat";
  static const String imageGenerateStatus = "El estado generar la imagen";
  static const String subscriptionPlanStatus = "Estado del plan de suscripción";
  static const String adShowStatus = "Estado de exhibición de anuncios";

  static const String apple = "Iniciar sesión con Apple";
  static const String updateProfile = "Actualización del perfil";

  static const String subscriptionWith = "Suscribirse";

  static const String changeTextModel = "Cambiar el modelo de texto";
  static const String setToken = "Token";
  static const String changeImageSize = "Tamaño de la imagen";
  static const String imageNote =
      "Los tamaños más pequeños son más rápidos de generar.";
  static const String fullName = "Nombre completo";
  static const String emailAddress = "Dirección de correo electrónico";
  static const String mobileNumber = "Número de teléfono móvil";

  static const String themeChange = "Cambio de tema";

  static const String myPlan = "Mi plan";
  static const String purchaseDate = "Fecha de compra";
  static const String logIn = "Acceso";

  static const String description = "Descripción";

  static const String writeContent = "Escribir contenido";

  static const String contentWriting = "Redacción de contenido";
  static const String contentWritingSubTitle =
      "Generate your content with OmAgent now! with powerful AI· Content Writer for all your content writing needs·";

  static const String selectType = "Seleccione tipo";
  static const String selectYourSocialMedia = "Seleccione sus redes sociales";
  static const String topic = "Tema";
  static const String about = "Acerca de";
  static const String forText = "Para";

  static const String hashTagNumber = "Número de etiqueta hash";
  static const String create = "Crear";
  static const String idea = "Idea";
  static const String copy = "Copiar";
  static const String hashTags = "Etiquetas";

  static const String keyWordCreating = "Palabra clave Generar";
  static const String hashTagCreatorSubTitle =
      "Genere sus hashtags con OmAgent ahora para sus redes sociales ·";

  static const String dietChartCreating = "Generar plan de dieta";
  static const String dietChartCreatorSubTitle =
      "Genera tu plan de dieta con OmAgent ahora para una vida saludable·";
  static const String currentWeight = "Peso actual (kg)";
  static const String targetWeight = "Peso objetivo (kg)";
  static const String height = "Altura (cm)";
  static const String dietPlan = "Plan de dieta";
  static const String gender = "Género";
  static const String dietDuration = "Duración de la dieta (semanas)";
  static const String lifeStyle = "Estilo de vida";
  static const String selectCountry =
      "Selecciona el país para la categoría de alimentos";
  static const String createDietChart =
      "Crear un plan de dieta para perder peso";
  static const String nowWeight = "Mi peso actual es";
  static const String kg = "kg";
  static const String cm = "cm";
  static const String weeks = "semanas";
  static const String expectedWeight = "Mi peso objetivo es";
  static const String heightIs = "Mi altura es";
  static const String myDietDuration = "Duración de mi dieta";
  static const String iAmA = "Soy";
  static const String myLifestyle = "Mi estilo de vida es";
  static const String basedOn = "Basado en";
  static const String food = "alimentos";
  static const String pdfHeader = "";

  static const String pressTheButton =
      "Presiona el botón para generar una nota de amor";
  static const String createARandomLoveNote =
      "Crea una cita de amor corta al azar para mi pareja, que no haya sido creada antes en este momento";

  static const String loveNotes = "Crear Notas de Amor";
  static const String randomLoveNotes = "Notas de Amor al Azar";
  static const String randomLoveNotesSubtitle =
      "Genera tus notas de amor con OmAgent ahora para una buena relación";

  static const String enterYourPassword = "Ingrese su contraseña";
  static const String go = "Ir";
  static const String forgotPassword = "¿Olvidó su contraseña?";

  /// 4.0.0

  static const String setSchedule = "Establecer Programación";
  static const String setScheduleReminder =
      "Establecer Recordatorio Programado";
  static const String setScheduleSubTitle =
      "Con nuestra función de programación, nunca más se perderá una tarea o evento. Establezca recordatorios fácilmente para reuniones, citas y elementos pendientes. Manténgase organizado y en control de su vida ocupada.";

  static const String scheduleName = "Nombre de la Programación";
  static const String enterScheduleName = "Ingrese $scheduleName";
  static const String scheduleTime = "Hora de la Programación";
  static const String scheduleDate = "Fecha de la Programación";
  static const String scheduleDescription = "Descripción de la Programación";

  static const String daily = "Diario";
  static const String weekly = "Semanal";
  static const String once = "Una Vez";

  static const String selectWeeks = "Seleccionar Semanas";

  static const String saturday = "Sábado";
  static const String sunday = "Domingo";
  static const String monday = "Lunes";
  static const String tuesday = "Martes";
  static const String wednesday = "Miércoles";
  static const String thursday = "Jueves";
  static const String friday = "Viernes";

  static const String fillUpForm =
      "Complete el Nombre y la Descripción Primero";

  static const String coverLetter = "Carta de presentación";
  static const String coverLetterCreatorSubTitle =
      "Crea tu carta de presentación con OmAgent ahora para una vida inteligente.";
  static const String date = "Fecha";
  static const String personalInformation = "Información personal";

  static const String address = "Dirección";
  static const String enterAddress = "Ingresa tu dirección";

  static const String enterMobileNumber = "Ingresa tu número de celular";

  static const String linkedInUrl = "URL de LinkedIn";
  static const String portfolioUrl = "URL del portafolio";
  static const String educationalBackground = "Antecedentes educativos";

  static const String newCompanyInformation = "Información de la nueva empresa";

  static const String jobTitle = "Cargo";
  static const String enterJobTitle = "Ingresa tu cargo";

  static const String companyName = "Nombre de la empresa";
  static const String enterCompanyName = "Ingresa el nombre de la empresa";

  static const String qualifications = "Cualificaciones";

  static const String skills = "Habilidades";
  static const String mentionSkills =
      "Menciona tus habilidades [Separadas por (,)]";

  static const String duration = "Duración";
  static const String totalExperience = "Experiencia total";
  static const String enterTotalExperience = "1 año";

  static const String latestWorkDescribe = "Descripción del último trabajo";
  static const String describeShortly = "Describe brevemente";

  static const String latestCompanyInformation =
      "Información de la última empresa";

  static const String myRole = "Mi rol";
  static const String enterMyRole = "Ingeniero de software junior";

  static const String others = "Otros";
  static const String pdf = "PDF";

  static const String doYouHaveWorkExperience = "¿Tienes experiencia laboral?";

  static const String hereYouCanModifyBeforeGettingPDFFile =
      "Aquí puedes modificar antes de obtener el archivo PDF";
}
