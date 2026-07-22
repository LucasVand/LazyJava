package processor;

import javax.annotation.processing.AbstractProcessor;
import javax.annotation.processing.RoundEnvironment;
import javax.annotation.processing.SupportedAnnotationTypes;
import javax.annotation.processing.SupportedSourceVersion;
import javax.lang.model.SourceVersion;
import javax.lang.model.element.Element;
import javax.lang.model.element.TypeElement;
import javax.tools.JavaFileObject;
import java.io.Writer;
import java.util.Set;

@SupportedAnnotationTypes("processor.MyAnnotation")
@SupportedSourceVersion(SourceVersion.RELEASE_8)
public class MyProcessor extends AbstractProcessor {

    @Override
    public boolean process(Set<? extends TypeElement> annotations, RoundEnvironment roundEnv) {
        if (roundEnv.processingOver()) {
            return false;
        }

        for (Element element : roundEnv.getElementsAnnotatedWith(
                processingEnv.getElementUtils().getTypeElement("processor.MyAnnotation"))) {
            try {
                JavaFileObject file = processingEnv.getFiler()
                        .createSourceFile("generated.MyGeneratedClass");
                try (Writer writer = file.openWriter()) {
                    writer.write(
                            "package generated;\n" +
                            "public class MyGeneratedClass {\n" +
                            "    public static String getMessage() {\n" +
                            "        return \"Hello from generated code!\";\n" +
                            "    }\n" +
                            "}"
                    );
                }
            } catch (Exception e) {
                processingEnv.getMessager().printMessage(
                        javax.tools.Diagnostic.Kind.ERROR, "Generation failed: " + e.getMessage());
            }
        }
        return true;
    }
}
