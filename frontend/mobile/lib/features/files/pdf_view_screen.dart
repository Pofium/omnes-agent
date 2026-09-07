
import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:get/get.dart';
import 'package:syncfusion_flutter_pdfviewer/pdfviewer.dart';

import '../../helper/download_file.dart';
import '../../utils/assets.dart';
import '../../utils/custom_color.dart';
import '../../utils/strings.dart';

class PdfViewScreen extends StatelessWidget with DownloadFile{

  final List<int> pdfBytes;
  final String name;

  PdfViewScreen({super.key, required this.pdfBytes, required this.name});

  @override
  Widget build(BuildContext context) {
    // final Uint8List bytbyteses = Uint8List.fromList(pdfBytes);

    return Scaffold(
      floatingActionButtonLocation: FloatingActionButtonLocation.endFloat,
      floatingActionButton: FloatingActionButton(
          tooltip: Strings.download.tr,
          onPressed: () async{
            try {

              await downloadFile2(pdfData: Uint8List.fromList(pdfBytes), name: name);

            } on PlatformException catch (error) {
              debugPrint(error.toString());
            }
          },
          child: const Icon(Icons.download)),
      appBar: AppBar(
        title:  Text(Strings.pdf.tr),
        backgroundColor: CustomColor.primaryColor.withOpacity(.30),
        actions: [
          Image.asset(
            Assets.bot,
            scale: 3,
          ),

          // IconButton(
          //   icon: const Icon(
          //     Icons.download,
          //     color: Colors.white,
          //   ),
          //   onPressed: () async {
          //     Directory directory = await getTemporaryDirectory();
          //     String path = directory.path;
          //     //Create the empty file.
          //     File file = File('$path/${'sample.pdf'}');
          //     //Write the PDF data retrieved from the SfPdfViewer.
          //     await file.writeAsBytes(bytes, flush: true);
          //     debugPrint(path);
          //               },
          // ),
        ],
      ),
      body: _bodyWidget(),
    );
  }

  Widget _bodyWidget() {
    final Uint8List bytes = Uint8List.fromList(pdfBytes);
    return SizedBox.expand(
      child: SfPdfViewer.memory(
        bytes,
      ),
    );
  }
}