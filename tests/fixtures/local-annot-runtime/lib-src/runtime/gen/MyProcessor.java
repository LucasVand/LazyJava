package runtime.gen;

import java.util.Set;
import javax.annotation.processing.AbstractProcessor;
import javax.annotation.processing.RoundEnvironment;
import javax.lang.model.SourceVersion;
import javax.lang.model.element.TypeElement;

public class MyProcessor extends AbstractProcessor {

    private boolean done;

    @Override
    public Set<String> getSupportedAnnotationTypes() {
        return Set.of("runtime.MyAnno");
    }

    @Override
    public SourceVersion getSupportedSourceVersion() {
        return SourceVersion.latestSupported();
    }

    @Override
    public boolean process(Set<? extends TypeElement> annotations, RoundEnvironment roundEnv) {
        if (roundEnv.processingOver() || done) {
            return false;
        }
        done = true;
        try {
            javax.tools.JavaFileObject file =
                    processingEnv.getFiler().createSourceFile("generated.GeneratedHello");
            try (java.io.Writer w = file.openWriter()) {
                w.write("package generated;\n");
                w.write("public class GeneratedHello {\n");
                w.write("    public static String msg() { return \"generated-by-processor\"; }\n");
                w.write("}\n");
            }
        } catch (java.io.IOException e) {
            throw new RuntimeException(e);
        }
        return false;
    }
}