package processor;

import javax.annotation.processing.*;
import javax.lang.model.SourceVersion;
import javax.lang.model.element.*;
import java.util.Set;

@SupportedAnnotationTypes("processor.Annot")
@SupportedSourceVersion(SourceVersion.RELEASE_21)
public class ProcessorImpl extends AbstractProcessor {

    @Override
    public boolean process(
            Set<? extends TypeElement> annotations,
            RoundEnvironment roundEnv) {

        System.out.println("Running processor!");

        return true;
    }
}
